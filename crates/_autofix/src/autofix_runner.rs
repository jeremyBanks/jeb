/// Run a list of autofixes sequentially.
/// Returns the exit code (0 if all succeed, otherwise first error code).
pub fn run_autofixes(modules: &[(&str, fn() -> i32)]) -> i32 {
    let mut first_error: Option<i32> = None;
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
    let total = modules.len();
    if failed_count == 0 {
        eprintln!("Ran {} autofixes (all successful)", total);
    } else if failed_count == total {
        eprintln!("Ran {} autofixes (all failed)", total);
    } else {
        eprintln!("Ran {} autofixes ({} failed)", total, failed_count);
    }
    first_error.unwrap_or(0)
}
