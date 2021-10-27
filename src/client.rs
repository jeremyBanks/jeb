#[derive(Debug)]
pub struct Client {
    pub http_client: reqwest::Client,
    pub request_interval: tokio::time::Interval,
}

impl Client {
    pub fn new() -> Self {
        let mut request_interval = tokio::time::interval(std::time::Duration::from_secs(1));
        request_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        Self {
            http_client: reqwest::Client::new(),
            request_interval,
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn fetch(&mut self, path: &str) -> eyre::Result<String> {
        self.request_interval.tick().await;

        let url = format!("https://stadia.com/{}", path);
        let request = self.http_client.get(url).build()?;
        let response = self.http_client.execute(request).await?;
        let body = response.text().await?;

        Ok(body)
    }
}
