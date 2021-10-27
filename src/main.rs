fn main() {
    // Initialize global unhandled error panic handler.
    color_eyre::install().expect("fatal error");

    // Initialize global logging handler.
    tracing_subscriber::util::SubscriberInitExt::init(tracing_subscriber::Layer::with_subscriber(
        tracing_error::ErrorLayer::default(),
        tracing_subscriber::fmt()
            .with_target(false)
            .with_span_events(
                tracing_subscriber::fmt::format::FmtSpan::NEW
                    | tracing_subscriber::fmt::format::FmtSpan::CLOSE,
            )
            .finish(),
    ));

    // Initialize the global async runtime.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("fatal error");

    // Start application.
    runtime.block_on(stadians::main())
}
