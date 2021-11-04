use std::time::Duration;

use eyre::{eyre, WrapErr};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tracing_unwrap::ResultExt;

pub struct Client {
    /// Internal HTTP client.
    http_client: reqwest::Client,
    /// Internal throttle used for all HTTP requests.
    http_throttle: tokio::time::Interval,

    api_cache: sled::Db,

    /// Google cookie header value for long-term authentication.
    cookie_header: Option<String>,
    /// Google API keys for short-term authentication.
    session_tokens: Option<SessionTokens>,
}

#[derive(Debug)]
struct SessionTokens {
    bl: String,
    f_sid: String,
    at: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiCall {
    pub rpc_id: String,
    pub request: Json,
    pub response: Option<Json>,
    pub timestamp: u64,
}

#[derive(Copy, Clone, Debug, PartialOrd, PartialEq, Serialize_repr, Deserialize_repr)]
#[repr(u32)]
enum ApiCallStatus {
    /// This call has been seeded into the database, but not attempted.
    Known = 0x00,
    /// This call has been attempted, but we don't know the result.
    Attempted = 0x10,
    /// This call failed for out-of-band reasons (i.e. network error, unexpected
    /// response format).
    Unable = 0x20,
    /// The call failed with an in-band error response value.
    Error = 0x30,
    /// The call succeeded with a successful but empty response value.
    Empty = 0x35,
    /// The call succeeded with a successful non-empty response value.
    Full = 0x40,
}

fn api_cache_key(method_id: &str, parameters: &Json, status: Option<ApiCallStatus>) -> Vec<u8> {
    let mut key = [0u8; 128];

    let key_prefix = "api_cache_".as_bytes();
    key[0..10].copy_from_slice(key_prefix);

    let key_method_id: [u8; 10] = fit_into_array(method_id.as_bytes());
    key[10..20].copy_from_slice(&key_method_id);

    let parameters_json = parameters.to_string();
    let key_parameters: [u8; 107] = fit_into_array(parameters_json.as_bytes());
    key[20..127].copy_from_slice(&key_parameters);

    let mut v = Vec::from(key);
    if let Some(status) = status {
        v.push(status as u8);
    }

    v
}

const USER_AGENT: &str = concat![
    "Mozilla/5.0 ",
    "(Windows NT 10.0; Win64; x64) ",
    "AppleWebKit/537.36 ",
    "(KHTML, like Gecko) ",
    "Chrome/95.0.4638.54 ",
    "Safari/537.36 ",
];

/// A URL of any Stadia SPA page containing session credentials.
const SPA_URL: &str = "https://stadia.google.com/settings";

/// The URL of the Stadia web frontend's API.
const API_URL: &str = "https://stadia.google.com/_/CloudcastPortalFeWebUi/data/batchexecute";

impl Client {
    pub fn new(cookie_header: Option<String>, api_cache: sled::Db) -> Self {
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
            .header("Cookie", self.cookie_header.clone().expect("no cookies"))
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
        let cache_key = api_cache_key(rpc_id, request, None);
        tracing::info!("API call: {}", printable(&cache_key, '_'));

        if let Some(Ok(cached)) = self.api_cache.scan_prefix(&cache_key).values().next_back() {
            tracing::info!(rpc_id, "API result found in cache");
            let cached: ApiCall = serde_json::from_slice(&cached.to_vec()).unwrap();
            return Ok(cached.response);
        } else {
            tracing::info!(rpc_id, "API result NOT found in cache, requesting it");
        }

        if self.session_tokens.is_none() {
            self.session_tokens = Some(self.get_session_tokens().await?);
        }
        let session_tokens = self.session_tokens.as_ref().unwrap();

        self.http_throttle.tick().await;

        let request_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let response = self
            .http_client
            .post(API_URL)
            .header("User-Agent", USER_AGENT)
            .header("Cookie", self.cookie_header.clone().expect("no cookies"))
            .query(&[
                ["bl", &session_tokens.bl],
                ["rpcids", rpc_id],
                ["f.sid", &session_tokens.f_sid],
                ["hl", "en"],
                ["_reqid", "123456"],
            ])
            .form(&[
                [
                    "f.req",
                    &(json!([[[rpc_id, request.to_string(), Json::Null, "1"]]]).to_string()),
                ],
                ["at", &session_tokens.at],
            ]);

        let response = response.send().await?;

        let prefixed_json = response.text().await?;
        let json = prefixed_json.strip_prefix(")]}'\n").unwrap();
        let value = json.parse::<Json>()?;

        tracing::info!(json);

        let response = value
            .as_array()
            .cloned()
            .ok_or_else(|| eyre!("expected JSON array"))?
            .iter()
            .map(|x| x[2].as_str().unwrap_or("[]").parse::<Json>().unwrap())
            .next()
            .unwrap();

        let response_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut cache_key = cache_key.to_vec();
        cache_key.extend_from_slice(&self.api_cache.generate_id().unwrap().to_be_bytes());

        self.api_cache
            .insert(
                cache_key.clone(),
                serde_json::to_vec(&ApiCall {
                    rpc_id: rpc_id.to_string(),
                    request: request.clone(),
                    response: Some(response.clone()),
                    timestamp: response_timestamp,
                })
                .unwrap(),
            )
            .expect("failed to save to cache?");

        self.api_cache.flush().expect("unable to flush cache?");

        Ok(response)
    }

    pub async fn player_search(&mut self, name_prefix: &str) -> eyre::Result<Json> {
        self.api_request("FdyJ0", &json!([name_prefix])).await
    }

    pub async fn store_search(&mut self, name_contains: &str) -> eyre::Result<Json> {
        self.api_request("QBe3Lb", &json!([name_contains])).await
    }
}

fn printable(bytes: &[u8], filler: char) -> String {
    regex::Regex::new(r"[^ -~]")
        .unwrap()
        .replace_all(
            &String::from_utf8_lossy(bytes).to_string(),
            filler.to_string(),
        )
        .to_string()
}

fn fit_into_array<const T: usize>(value: &[u8]) -> [u8; T] {
    let mut array = [0x00; T];

    if value.len() <= T {
        // If the value fits in the array, great, put it in.
        // If it's shorter than the array, we'll leave trailing zero-bytes.
        array[..value.len()].copy_from_slice(value);
    } else {
        // If the value's too large to fit in the array, hash it with blake3.
        let mut hasher = blake3::Hasher::new();
        hasher.update(value);
        let mut result = hasher.finalize_xof();

        // Fill the first half of the array with value, truncated to fit.
        array[..T / 2].copy_from_slice(&value[..T / 2]);
        // Fill the the second half from the hash digest.
        result.fill(&mut array[T / 2..]);
    }

    array
}
