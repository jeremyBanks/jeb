use std::time::Duration;

use eyre::{eyre, WrapErr};
use serde_json::json;

#[derive(Debug)]
pub struct Client {
    /// Internal HTTP client.
    http_client: reqwest::Client,
    /// Internal throttle used for all HTTP requests.
    http_throttle: tokio::time::Interval,
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

// The anti-competitive monopolists at Google filter user agents, so we lie.
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

const API_URL: &str = "https://stadia.google.com/u/0/_/CloudcastPortalFeWebUi/data/batchexecute";

impl Client {
    #[tracing::instrument]
    pub fn new(cookie_header: String) -> Self {
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
            cookie_header: std::env::var("GOOGLE_SID")
                .map(|sid| format!("GOOGLE_SID={}", sid))
                .wrap_err_with(|| eyre!("expected GOOGLE_SID environment variable"))
                .unwrap(),
            session_tokens: None,
        }
    }

    #[tracing::instrument(skip(self))]
    async fn get_session_tokens(&mut self) -> eyre::Result<SessionTokens> {
        self.http_throttle.tick().await;

        let response = self
            .http_client
            .get(SPA_URL)
            .header("User-Agent", USER_AGENT)
            .header("Cookie", &self.cookie_header)
            .send()
            .await?;
        let html = response.text().await?;

        unimplemented!()
    }

    #[tracing::instrument(skip(self))]
    pub async fn api_request(
        &mut self,
        requests: &[(&str, serde_json::Value)],
    ) -> eyre::Result<Vec<serde_json::Value>> {
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
                ["rpcids", requests[0].0],
                ["rt", "j"],
                ["hl", "en"],
                ["f.sid", &session_tokens.f_sid],
                ["_reqid", "123456"],
            ])
            .form(&[
                ["f.req", &(json!([requests[0].1,]).to_string())],
                ["at", &session_tokens.at],
            ])
            .send()
            .await?;
        let prefixed_json = response.text().await?;
        let json = prefixed_json.strip_prefix(")]}'\n").unwrap();
        let value = json.parse::<serde_json::Value>()?;
        value
            .as_array()
            .ok_or_else(|| eyre!("expected JSON array"))
            .map(Clone::clone)
    }
}
