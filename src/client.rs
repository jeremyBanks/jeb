use eyre::{eyre, WrapErr};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use std::time::Duration;
use tracing_unwrap::ResultExt;

pub struct Client {
    /// Internal HTTP client.
    http_client: reqwest::Client,
    /// Internal throttle used for all HTTP requests.
    http_throttle: tokio::time::Interval,

    api_cache: ApiCacheBucket,

    /// Google cookie header value for long-term authentication.
    cookie_header: String,
    /// Google API keys for short-term authentication.
    session_tokens: Option<SessionTokens>,
}

#[derive(Debug)]
struct SessionTokens {
    bl: String,
    f_sid: String,
    at: String,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct ApiCall {
    pub rpc_id: String,
    pub request: Json,
    pub response: Json,
    pub request_timestamp_ms: u64,
    pub response_timestamp_ms: u64,
}

pub type ApiCacheBucket = kv::Bucket<'static, String, kv::Json<ApiCall>>;

const USER_AGENT: &str = concat![
    "Mozilla/5.0 ",
    "(Windows NT 10.0; Win64; x64) ",
    "AppleWebKit/537.36 ",
    "(KHTML, like Gecko) ",
    "Chrome/95.0.4638.54 ",
    "Safari/537.36 ",
];

/// A URL of any Stadia SPA page containing session credentials.
const SPA_URL: &str = "https://stadia.google.com/u/0/settings";

/// The URL of the Stadia web frontend's API.
const API_URL: &str = "https://stadia.google.com/u/0/_/CloudcastPortalFeWebUi/data/batchexecute";

impl Client {
    pub fn new(cookie_header: String, api_cache: ApiCacheBucket) -> Self {
        let http_client = reqwest::Client::builder()
            .user_agent(USER_AGENT)
            .timeout(Duration::from_secs(12))
            .build()
            .expect("failed to initialize HTTP client");

        let mut http_throttle = tokio::time::interval(Duration::from_secs(1));
        http_throttle.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        Self {
            http_client,
            http_throttle,
            api_cache,
            cookie_header,
            session_tokens: None,
        }
    }

    async fn get_session_tokens(&mut self) -> eyre::Result<SessionTokens> {
        self.http_throttle.tick().await;
        tracing::info!("Fetching session tokens");

        let response = self
            .http_client
            .get(SPA_URL)
            .header("User-Agent", USER_AGENT)
            .header("Cookie", &self.cookie_header)
            .send()
            .await?;
        let html = response.text().await?;
        let wiz_global_data = html
            .split_once("WIZ_global_data =")
            .ok_or_else(|| eyre!("WIZ_global_data not found in SPA page"))?
            .1
            .split_once(";</script>")
            .ok_or_else(|| eyre!("closing </script> not found after WIZ_global_data"))?
            .0;
        let wiz_global_json = wiz_global_data
            .parse::<Json>()?
            .as_object()
            .ok_or_else(|| eyre!("WIZ_global_data is not a JSON object"))?
            .clone();

        Ok(SessionTokens {
            bl: wiz_global_json
                .get("cfb2h")
                .ok_or_else(|| eyre!("`cfb2h` for `bl` not found in WIZ_global_data"))?
                .as_str()
                .ok_or_else(|| eyre!("`cfb2h` for `bl` was not a string"))?
                .to_string(),
            f_sid: wiz_global_json
                .get("FdrFJe")
                .ok_or_else(|| eyre!("`FdrFJe` for `f_sid` not found in WIZ_global_data"))?
                .as_str()
                .ok_or_else(|| eyre!("`FdrFJe` for `f_sid` was not a string"))?
                .to_string(),
            at: wiz_global_json
                .get("SNlM0e")
                .ok_or_else(|| eyre!("`SNlM0e` for `at` not found in WIZ_global_data"))?
                .as_str()
                .ok_or_else(|| eyre!("`SNlM0e` for `at` was not a string"))?
                .to_string(),
        })
    }

    pub async fn api_request(&mut self, rpc_id: &str, request: &Json) -> eyre::Result<Json> {
        let cache_key = format!("{}{}", rpc_id, request.to_string());
        tracing::info!("API call: {}", &cache_key);

        if let Ok(Some(cached)) = self.api_cache.get(&cache_key) {
            tracing::info!(rpc_id, "API result found in cache");
            return Ok(cached.0.response);
        } else {
            tracing::info!(rpc_id, "API result NOT found in cache, requesting it");
        }

        if self.session_tokens.is_none() {
            self.session_tokens = Some(self.get_session_tokens().await?);
        }
        let session_tokens = self.session_tokens.as_ref().unwrap();

        self.http_throttle.tick().await;
        let response = self
            .http_client
            .post(API_URL)
            .header("User-Agent", USER_AGENT)
            .header("Cookie", &self.cookie_header)
            .query(&[
                ["bl", &session_tokens.bl],
                ["rpcids", rpc_id],
                ["hl", "en"],
                ["f.sid", &session_tokens.f_sid],
            ])
            .form(&[
                [
                    "f.req",
                    &(Json::from(vec![json!([rpc_id, request.to_string(), Json::Null, "1"])])
                        .to_string()),
                ],
                ["at", &session_tokens.at],
            ])
            .send()
            .await?;
        let prefixed_json = response.text().await?;
        let json = prefixed_json.strip_prefix(")]}'\n").unwrap();
        let value = json.parse::<Json>()?;

        let response = value
            .as_array()
            .cloned()
            .ok_or_else(|| eyre!("expected JSON array"))?
            .iter()
            .map(|x| x[2].as_str().unwrap_or("[]").parse::<Json>().unwrap())
            .next()
            .unwrap();

        self.api_cache
            .set(
                cache_key.clone(),
                kv::Json(ApiCall {
                    rpc_id: rpc_id.to_string(),
                    request: request.clone(),
                    response: response.clone(),
                    request_timestamp_ms: 0,
                    response_timestamp_ms: 0,
                }),
            )
            .expect("failed to save to cache?");

        self.api_cache.flush().expect("unable to flush cache?");

        Ok(response)
    }
}
