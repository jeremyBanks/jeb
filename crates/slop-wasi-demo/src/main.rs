use {
    serde_json::Value,
    std::io::{self, Read, Write},
};

/// Simple JSON pretty-printer demo for WASI/Deno distribution
///
/// This demonstrates WASI-compatible patterns:
/// - Single-threaded tokio runtime (current_thread) - available but not needed here
/// - std::fs and std::io for all I/O operations (WASI compatible)
/// - No tokio::io::stdin/stdout (not supported on WASI)
///
/// Usage:
///   slop-wasi-demo                    # Read from stdin
///   slop-wasi-demo input.json         # Read from file
///   slop-wasi-demo input.json --compact  # Compact output

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Note: We're using tokio::main but not actually using any async features
    // This demonstrates that the runtime works on WASI, even if we don't need it here
    run_sync()
}

fn run_sync() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    // Parse arguments
    let (input_path, compact) = parse_args(&args);

    // Read JSON - all I/O is synchronous (WASI appropriate)
    let json_text = if let Some(path) = input_path {
        eprintln!("Reading from file: {}", path);
        read_file(&path)?
    } else {
        eprintln!("Reading from stdin...");
        read_stdin()?
    };

    // Parse JSON
    let value: Value = serde_json::from_str(&json_text)
        .map_err(|e| format!("JSON parse error: {}", e))?;

    // Format output
    let output = if compact {
        serde_json::to_string(&value)?
    } else {
        serde_json::to_string_pretty(&value)?
    };

    // Write to stdout
    write_stdout(&output)?;

    Ok(())
}

fn parse_args(args: &[String]) -> (Option<String>, bool) {
    let mut input_path = None;
    let mut compact = false;

    for arg in args.iter().skip(1) {
        if arg == "--compact" || arg == "-c" {
            compact = true;
        } else if arg == "--help" || arg == "-h" {
            print_help();
            std::process::exit(0);
        } else if !arg.starts_with('-') && input_path.is_none() {
            input_path = Some(arg.clone());
        }
    }

    (input_path, compact)
}

fn print_help() {
    eprintln!("slop-wasi-demo - JSON pretty-printer for WASI/Deno");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("    slop-wasi-demo [FILE] [OPTIONS]");
    eprintln!();
    eprintln!("ARGS:");
    eprintln!("    <FILE>    Input JSON file (default: stdin)");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("    -c, --compact    Output compact JSON");
    eprintln!("    -h, --help       Print this help message");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("    echo '{{\"hello\":\"world\"}}' | slop-wasi-demo");
    eprintln!("    slop-wasi-demo input.json");
    eprintln!("    slop-wasi-demo input.json --compact");
}

/// Read file using std::fs (WASI compatible)
fn read_file(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = std::fs::File::open(path)
        .map_err(|e| format!("Failed to open {}: {}", path, e))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| format!("Failed to read {}: {}", path, e))?;
    Ok(contents)
}

/// Read stdin using std::io (WASI compatible)
fn read_stdin() -> Result<String, Box<dyn std::error::Error>> {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    Ok(buffer)
}

/// Write to stdout using std::io (WASI compatible)
fn write_stdout(text: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = io::stdout();
    stdout.write_all(text.as_bytes())?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    Ok(())
}
