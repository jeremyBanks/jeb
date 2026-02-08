//! Colored command output formatting similar to `./run`.

use jeb_tracing::use_color;

// ANSI escape codes for colors
const RESET: &str = "\x1b[0m";
const DARK_BACKGROUND: &str = "\x1b[40m\x1b[K";
const WHITE: &str = "\x1b[37m";
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";
const BLUE: &str = "\x1b[34m";

/// Print a command in the styled format used by `./run`.
///
/// Format:
/// ```text
///   $ program arg1 arg2
/// ```
///
/// With colors: magenta `$`, cyan program, blue args, on dark background.
pub fn print_command(program: &str, args: &[&str]) {
    if use_color() {
        eprintln!("{DARK_BACKGROUND}");
        if args.is_empty() {
            eprintln!("{DARK_BACKGROUND}  {MAGENTA}${WHITE} {CYAN}{program}{WHITE}");
        } else {
            let args_str = args.join(" ");
            eprintln!(
                "{DARK_BACKGROUND}  {MAGENTA}${WHITE} {CYAN}{program}{WHITE} \
                 {BLUE}{args_str}{WHITE}"
            );
        }
        eprintln!("{DARK_BACKGROUND}{RESET}");
    } else {
        if args.is_empty() {
            eprintln!("  $ {program}");
        } else {
            eprintln!("  $ {program} {}", args.join(" "));
        }
    }
    eprintln!();
}
