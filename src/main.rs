#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize global unhandled error panic handler.
    color_eyre::install()?;

    // Initialize global logging handler.
    tracing_subscriber::util::SubscriberInitExt::init(tracing_subscriber::Layer::with_subscriber(
        tracing_error::ErrorLayer::default(),
        tracing_subscriber::fmt()
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ACTIVE)
            .finish(),
    ));

    log::info!("hello, log!");
    tracing::info!("hello, tracing!");

    Ok(())
}
