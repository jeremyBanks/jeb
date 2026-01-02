use _autofix::{
    autofix_runner,
    cargo_clippy,
    cargo_fix,
    cargo_fmt,
    deno_fmt,
    deno_lint,
    workspace_deps,
};
fn main() {
    let modules: &[(&str, fn() -> i32)] = &[
        ("cargo_fmt", cargo_fmt::main),
        ("cargo_fix", cargo_fix::main),
        ("cargo_clippy", cargo_clippy::main),
        ("workspace_deps", workspace_deps::main),
        ("deno_lint", deno_lint::main),
        ("deno_fmt", deno_fmt::main),
    ];
    let exit_code = autofix_runner::run_autofixes(modules);
    std::process::exit(exit_code);
}
