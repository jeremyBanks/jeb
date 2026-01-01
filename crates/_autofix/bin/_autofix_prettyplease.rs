// Note: This binary only runs the prettyplease autofix
// Use this to format code without running other autofixes

// Import modules from the lib (we need to expose them as lib)
use _autofix::{autofix_runner, prettyplease};

fn main() {
    // Only run prettyplease
    let modules: &[(&str, fn() -> i32)] = &[("prettyplease", prettyplease::main)];

    let exit_code = autofix_runner::run_autofixes(modules);
    std::process::exit(exit_code);
}
