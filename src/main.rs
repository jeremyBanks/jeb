#[allow(unused_imports)]
use tracing::{
    debug, debug_span, error, error_span, info, info_span, instrument, span, trace, trace_span,
    warn, warn_span,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    color_eyre::install()?;
    tracing_subscriber::util::SubscriberInitExt::init(tracing_subscriber::Layer::with_subscriber(
        tracing_error::ErrorLayer::default(),
        tracing_subscriber::fmt()
            .with_span_events(tracing_subscriber::fmt::format::FmtSpan::ACTIVE)
            .finish(),
    ));

    info!("Hello, world!");

    let x = get_a_number();

    info!(x, "Got a number!");

    Ok(())
}

#[instrument(level = "info")]
fn get_a_number() -> i32 {
    do_some_math(3)
}

#[instrument(level = "debug")]
fn do_some_math(n: i32) -> i32 {
    let base = {
        let _ = info_span!("doing some difficult math").enter();

        "3".parse::<i32>().unwrap()
    };

    base * n
}
