#[tracing::instrument]
pub async fn main() {
    log::info!("hello, log!");
    tracing::info!("hello, tracing!");

    something().await.expect("request failed");
}

async fn something() -> reqwest::Result<()> {
    let m = reqwest::get("https://github.com").await?.text().await?;
    let length = m.len();
    tracing::info!(length, "we got some bytes!");

    Ok(())
}
