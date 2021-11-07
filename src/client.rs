use std::time::Duration;

use derive_more::{From, Into, TryInto};
use eyre::{eyre, WrapErr};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};
use serde_repr::{Deserialize_repr, Serialize_repr};

pub mod protos;

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

fn api_cache_key(method_id: &str, parameters: &Json) -> Vec<u8> {
    let mut key = [0u8; 128];

    let key_prefix = "api_cache_".as_bytes();
    key[0..10].copy_from_slice(key_prefix);

    let key_method_id: [u8; 10] = fit_into_array(method_id.as_bytes());
    key[10..20].copy_from_slice(&key_method_id);

    let parameters_json = parameters.to_string();
    let key_parameters: [u8; 107] = fit_into_array(parameters_json.as_bytes());
    key[20..127].copy_from_slice(&key_parameters);

    key.to_vec()
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

        let mut http_throttle = tokio::time::interval(Duration::from_secs_f64(1.0));
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
        let cache_key = api_cache_key(rpc_id, request);
        tracing::debug!("API call: {}", printable(&cache_key, ' ').trim());

        if let Some(Ok(cached)) = self.api_cache.scan_prefix(&cache_key).values().next_back() {
            tracing::debug!(rpc_id, "API result found in cache");
            let cached: ApiCall = serde_json::from_slice(&cached.to_vec()).unwrap();
            if let Some(response) = cached.response {
                if response.is_null() {
                    tracing::info!(
                        rpc_id,
                        "but it it's null: {}",
                        printable(&cache_key, ' ').trim()
                    );
                // } else if let Some(response) = response.as_array() {
                //     if response.is_empty() {
                //         tracing::info!(
                //             rpc_id,
                //             "but it's empty: {}",
                //             printable(&cache_key, ' ').trim()
                //         );
                //     } else {
                //         return Ok(json!(response));
                //     }
                } else {
                    return Ok(response);
                    tracing::error!("wtf?");
                }
            } else {
                tracing::info!(
                    rpc_id,
                    "but it doesn't have a response: {}",
                    printable(&cache_key, ' ').trim()
                );
            }
        } else {
            tracing::info!(
                rpc_id,
                "API result NOT found in cache, requesting it: {}",
                printable(&cache_key, ' ').trim()
            );
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

    pub async fn player_search(
        &mut self,
        name_prefix: &str,
    ) -> eyre::Result<protos::PlayerSearchResponse> {
        let name_prefix = format!("{} {}", &name_prefix[..1], &name_prefix[1..]);

        let response = self.api_request("FdyJ0", &json!([name_prefix])).await?;
        tracing::debug!("{}", &response.to_string()[..100]);

        Ok(serde_json::from_str::<protos::PlayerSearchResponse>(
            &response.to_string(),
        )?)
    }

    pub async fn store_search(&mut self, name_contains: &str) -> eyre::Result<Vec<SearchSku>> {
        let response = self.api_request("QBe3Lb", &json!([name_contains])).await;
        let response = response.unwrap();
        let response = response.as_array().unwrap().get(1).unwrap();
        let response = response
            .as_array()
            .unwrap()
            .iter()
            .filter(|d| d[1][2][0].as_str().unwrap().ends_with(".Card"))
            .map(|d| d[1][2][1].clone())
            .map(|d| SearchSku {
                name: d[1].as_str().unwrap().to_string(),
                sku_id: d[9].as_array().unwrap()[0].as_array().unwrap()[1]
                    .as_str()
                    .unwrap()
                    .to_string(),
                game_id: d[9].as_array().unwrap()[0].as_array().unwrap()[0]
                    .as_str()
                    .unwrap()
                    .to_string(),
            })
            .collect::<Vec<_>>();
        Ok(response)
    }

    pub async fn store_sku(
        &mut self,
        sku_id: &str,
    ) -> eyre::Result<Option<protos::StoreSkuResponse>> {
        let response = self.api_request("FWhQV", &json!([null, sku_id])).await;
        let response = response.unwrap();
        tracing::debug!("{}", &response.to_string()[..100]);

        Ok(serde_json::from_str::<Option<protos::StoreSkuResponse>>(
            &response.to_string(),
        )?)
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SearchSku {
    pub game_id: String,
    pub sku_id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StoreSku {
    pub sku_type: SkuType,
    pub sku_id: String,
    pub game_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
}

impl StoreSku {
    pub fn from_proto(proto: &Json) -> Self {
        let proto = proto.as_array().unwrap();
        StoreSku {
            sku_type: proto[6].as_u64().unwrap().try_into().unwrap(),
            sku_id: proto[0].as_str().unwrap().to_string(),
            game_id: proto[4].as_str().map(|s| s.to_string()),
            name: proto[1].as_str().unwrap().to_string(),
            description: proto[9].as_str().map(|s| s.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SkuType {
    Game = 1,
    Addon = 2,
    Bundle = 3,
    ExternalSubscription = 4,
    StadiaSubscription = 5,
    AddonSubscription = 6,
    AddonBundle = 9,
    PreorderBundle = 10,
}

impl TryFrom<u64> for SkuType {
    type Error = eyre::Error;

    fn try_from(value: u64) -> Result<Self, eyre::Error> {
        match value {
            1 => Ok(SkuType::Game),
            2 => Ok(SkuType::Addon),
            3 => Ok(SkuType::Bundle),
            4 => Ok(SkuType::ExternalSubscription),
            5 => Ok(SkuType::StadiaSubscription),
            6 => Ok(SkuType::AddonSubscription),
            9 => Ok(SkuType::AddonBundle),
            10 => Ok(SkuType::PreorderBundle),
            _ => Err(eyre::eyre!("Unknown sku type: {}", value)),
        }
    }
}
