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
    let default = "warn,jeb=info,save=info".to_string();

    specific.or(common).unwrap_or(default)
});

pub static PANIC_INITIALIZED: LazyLock<bool> = LazyLock::new(|| {
    if !ENABLED.get_or_init(|| true) {
        return false;
    }

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

    true
});

#[macro_export]
macro_rules! trace {
    ($($args:tt)*) => {
        TRACING_INITIALIZED.get().ok();
        tracing::trace!($($args)*);
    };
}
