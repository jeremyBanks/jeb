mod command_runner;
mod cargo_fmt;
mod cargo_fix;
mod cargo_clippy;
mod workspace_deps;
mod deno_lint;
mod deno_fmt;

fn main() {
    let mut first_error: Option<i32> = None;

    // Run all autofixes sequentially
    let modules: &[(&str, fn() -> i32)] = &[
        ("cargo_fmt", cargo_fmt::main),
        ("cargo_fix", cargo_fix::main),
        ("cargo_clippy", cargo_clippy::main),
        ("workspace_deps", workspace_deps::main),
        ("deno_lint", deno_lint::main),
        ("deno_fmt", deno_fmt::main),
    ];

    let mut failed_count = 0;

    for (name, func) in modules {
        let code = func();
        if code != 0 {
            eprintln!("[{}] returned exit code: {}", name, code);
            if first_error.is_none() {
                first_error = Some(code);
            }
            failed_count += 1;
        }
    }

    // Print summary
    let total = modules.len();
    if failed_count == 0 {
        eprintln!("Ran {} autofixes (all successful)", total);
    } else if failed_count == total {
        eprintln!("Ran {} autofixes (all failed)", total);
    } else {
        eprintln!("Ran {} autofixes ({} failed)", total, failed_count);
    }

    // Exit with first error code, or 0 if all succeeded
    std::process::exit(first_error.unwrap_or(0));
}
