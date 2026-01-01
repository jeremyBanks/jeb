use std::process::{Command, Stdio};

/// Run a command, logging it first, letting stdio pass through.
/// Returns the exit code (0 = success).
pub fn run_command(program: &str, args: &[&str]) -> i32 {
    // Log the command
    let cmd_str = if args.is_empty() {
        program.to_string()
    } else {
        format!("{} {}", program, args.join(" "))
    };
    eprintln!("Running: {}", cmd_str);

    // Run the command
    match Command::new(program)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
    {
        Ok(status) => {
            let code = status.code().unwrap_or(1);
            eprintln!();  // Blank line after command
            code
        }
        Err(e) => {
            eprintln!("Error running command: {}", e);
            eprintln!();  // Blank line after error
            1
        }
    }
}
