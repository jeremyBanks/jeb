//! A minimal wrapper of the `tracing` crate, which just re-exports wrappers of
//! all of their key macros, inserting a call to a global function which
//! initializes our default tracing setup if no other tracing setup has been
//! configured, and our default tracing setup hasn't been explicitly disabled.

use std::{
    io::IsTerminal,
    sync::{
        LazyLock,
        OnceLock,
    },
};

#[doc(hidden)]
pub use tracing as __tracing;

pub static ENABLED: OnceLock<bool> = OnceLock::new();

/// Checks if an environment variable is set to a truthy value.
///
/// Returns `true` if the variable is set to any value except "0" or "false"
/// (case-sensitive). An empty string is considered truthy.
fn env_is_truthy(name: &str) -> bool {
    match std::env::var(name) {
        Ok(val) => val != "0" && val != "false",
        Err(_) => false,
    }
}

/// Returns whether colored output should be used.
///
/// Precedence:
/// 1. `NO_COLOR` set and truthy → disable color
/// 2. `FORCE_COLOR` set and truthy → enable color
/// 3. Otherwise → enable if stderr is a TTY
///
/// See <https://no-color.org/> and <https://force-color.org/>.
pub fn use_color() -> bool {
    if env_is_truthy("NO_COLOR") {
        return false;
    }
    if env_is_truthy("FORCE_COLOR") {
        return true;
    }
    std::io::stderr().is_terminal()
}

pub static LOG_ENV: LazyLock<String> = LazyLock::new(|| {
    let common = std::env::var("RUST_LOG");
    let specific = std::env::var("JEB_LOG");

    let default_log_level = "warn";
    let verbose_log_level = "info";

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

    let mut default_parts = Vec::<String>::new();

    default_parts.push(default_log_level.to_string());

    for crate_name in verbose_crates {
        default_parts.push(format!("{crate_name}={verbose_log_level}"));
    }

    let default = default_parts.join(",");

    specific.or(common).unwrap_or(default)
});

pub static PANIC_INITIALIZED: LazyLock<bool> = LazyLock::new(|| {
    if !ENABLED.get_or_init(|| true) {
        return false;
    }

    info!("Initialized default `jeb-tracing` configuration for panic handling.");

    let builder = color_eyre::config::HookBuilder::default();
    let builder = if use_color() {
        builder
    } else {
        builder.theme(color_eyre::config::Theme::new())
    };
    builder.install().is_ok()
});

pub static TRACING_INITIALIZED: LazyLock<bool> = LazyLock::new(|| {
    if !ENABLED.get_or_init(|| true) {
        return false;
    }

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(LOG_ENV.as_str()))
        .with_ansi(use_color())
        .pretty()
        .init();

    tracing::debug!(
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
                        $crate::__tracing::$ident!($D ($D args)*)
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
