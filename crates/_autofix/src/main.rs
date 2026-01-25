use {
    _autofix::{
        autofix_runner,
        cargo_clippy,
        cargo_fix,
        cargo_fmt,
        command_runner::run_command,
        deno_fmt,
        deno_lint,
    },
    tracing_subscriber::filter::EnvFilter,
};

fn main() {
    // Initialize tracing with env-filter
    // Default: warn for external crates, debug for this crate
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("_autofix=debug,warn"));

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_writer(std::io::stderr)
        .init();

    let modules: &[(&str, fn() -> i32)] = &[
        ("cargo_fmt", cargo_fmt::main),
        ("cargo_fix", cargo_fix::main),
        ("cargo_clippy", cargo_clippy::main),
        ("workspace_deps", || run_command("./run", &["workspace-deps"])),
        ("cargo_toml_normalize", || {
            run_command("./run", &["cargo-toml-normalize"])
        }),
        ("deno_lint", deno_lint::main),
        ("deno_fmt", deno_fmt::main),
    ];
    let exit_code = autofix_runner::run_autofixes(modules);
    std::process::exit(exit_code);
}
