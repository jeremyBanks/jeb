//! A minimal wrapper of the `tracing` crate, which just re-exports wrappers of
//! all of their key macros, inserting a call to a global function which
//! initializes our default tracing setup if no other tracing setup has been
//! configured, and our default tracing setup hasn't been explicitly disabled.

use std::sync::{
    LazyLock,
    OnceLock,
};

pub static ENABLED: OnceLock<bool> = OnceLock::new();

pub static LOG_ENV: LazyLock<String> = LazyLock::new(|| {
    let common = std::env::var("RUST_LOG");
    let specific = std::env::var("JEB_LOG");

    let default_log_level = "warn";

    let verbose_crates = [
        option_env!("CARGO_CRATE_NAME"),
        option_env!("CARGO_PKG_NAME"),
        option_env!("CARGO_BIN_NAME"),
        Some("jeb"),
        Some("save"),
        Some("inline"),
        Some("_"),
        Some("jeb-"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<&str>>();

    let default = "warn,jeb=info,save=info".to_string();

    specific.or(common).unwrap_or(default)
});

pub static PANIC_INITIALIZED: LazyLock<bool> = LazyLock::new(|| {
    if !ENABLED.get_or_init(|| true) {
        return false;
    }

    info!("Initialized default `jeb-tracing` configuration for panic handling.");

    color_eyre::install().is_ok()
});

pub static TRACING_INITIALIZED: LazyLock<bool> = LazyLock::new(|| {
    if !ENABLED.get_or_init(|| true) {
        return false;
    }

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(LOG_ENV.as_str()))
        .pretty()
        .init();

    tracing::info!(
        "Initialized default `jeb-tracing` configuration for global default `tracing` subscriber."
    );

    true
});

macro_rules! define_tracing_macro_wrappers {
    ( [$D:tt] $($ident:ident),+ $(,)?) => {
        $(
            #[macro_export]
            macro_rules! $ident {
                ($D ($D args:tt)*) => {
                    {
                        ::std::sync::LazyLock::force(&$crate::TRACING_INITIALIZED);
                        ::tracing::$ident!($D ($D args)*)
                    }
                };
            }
        )+
    };
}

define_tracing_macro_wrappers! {
    [$] span, enabled, record_all,
    trace, debug, info, warn, error,
    trace_span, debug_span, info_span, warn_span, error_span,
}
