mod client;
mod spider;

#[tracing::instrument(name = "stadians")]
pub async fn main() {
    log::info!("hello, log!");
    tracing::info!("hello, tracing!");

    let mut spider = crate::spider::Spider::default();
    let result = spider.client.fetch("settings").await.unwrap();
    let length = result.len();
    tracing::info!(length, "we got some bytes!");
}
