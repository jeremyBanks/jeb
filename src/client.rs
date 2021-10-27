#[derive(Default, Debug)]
pub struct Client {
    pub http_client: reqwest::Client,
}

impl Client {
    #[tracing::instrument(skip(self))]
    pub async fn fetch(&mut self, path: &str) -> eyre::Result<String> {
        let url = format!("https://stadia.com/{}", path);
        let request = self.http_client.get(url).build()?;
        let response = self.http_client.execute(request).await?;
        let body = response.text().await?;
        Ok(body)
    }
}
