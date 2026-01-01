use _autofix::{autofix_runner, prettyplease};
fn main() {
    let modules: &[(&str, fn() -> i32)] = &[("prettyplease", prettyplease::main)];
    let exit_code = autofix_runner::run_autofixes(modules);
    std::process::exit(exit_code);
}
