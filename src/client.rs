use std::time::Duration;

use crate::{credentials, errors::*};

#[derive(Debug)]
pub struct Client {
    credentials: credentials::GoogleCookies,
    http_client: reqwest::Client,
    http_throttle: tokio::time::Interval,
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

const API_ENDPOINT: &str = concat![
    "https://stadia.google.com/",
    "_/CloudcastPortalFeWebUi/data/batchexecute",
];

impl Client {
    #[tracing::instrument]
    pub fn new(credentials: credentials::GoogleCookies) -> Self {
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
            credentials,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn http_post(&mut self, path: &str) -> Result<String> {
        self.http_throttle.tick().await;

        let url = format!("https://stadia.com/{}", path);
        let request = self.http_client.post(url).build()?;
        let response = self.http_client.execute(request).await?;
        let body = response.text().await?;

        Ok(body)
    }
}
