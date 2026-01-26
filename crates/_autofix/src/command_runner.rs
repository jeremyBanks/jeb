use {
    crate::format::print_command,
    std::process::{
        Command,
        Stdio,
    },
};

/// Run a command, logging it first, letting stdio pass through.
/// Returns the exit code (0 = success).
pub fn run_command(program: &str, args: &[&str]) -> i32 {
    print_command(program, args);
    match Command::new(program)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
    {
        Ok(status) => {
            let code = status.code().unwrap_or(1);
            eprintln!();
            code
        }
        Err(e) => {
            eprintln!("Error running command: {}", e);
            eprintln!();
            1
        }
    }
}
