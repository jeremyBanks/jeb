mod z85;

use std::io::{self, Read, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("error: expected exactly one argument: 'encode' or 'decode'");
        eprintln!("Usage: {} <encode|decode>", args.get(0).map(|s| s.as_str()).unwrap_or("z85"));
        return ExitCode::FAILURE;
    }

    let command = &args[1];

    match command.as_str() {
        "encode" => {
            // Read all bytes from stdin
            let mut input = Vec::new();
            if let Err(e) = io::stdin().read_to_end(&mut input) {
                eprintln!("error: failed to read stdin: {}", e);
                return ExitCode::FAILURE;
            }

            // Encode to Z85
            let encoded = z85::encode(&input);

            // Write to stdout
            if let Err(e) = io::stdout().write_all(encoded.as_bytes()) {
                eprintln!("error: failed to write to stdout: {}", e);
                return ExitCode::FAILURE;
            }

            ExitCode::SUCCESS
        }
        "decode" => {
            // Read all bytes from stdin
            let mut input = Vec::new();
            if let Err(e) = io::stdin().read_to_end(&mut input) {
                eprintln!("error: failed to read stdin: {}", e);
                return ExitCode::FAILURE;
            }

            // Convert input to string (Z85 is ASCII, so this should be valid UTF-8)
            let input_str = match std::str::from_utf8(&input) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: invalid UTF-8 in input: {}", e);
                    return ExitCode::FAILURE;
                }
            };

            // Decode from Z85
            let decoded = match z85::decode(input_str) {
                Ok(bytes) => bytes,
                Err(e) => {
                    eprintln!("error: {}", e);
                    return ExitCode::FAILURE;
                }
            };

            // Write raw bytes to stdout
            if let Err(e) = io::stdout().write_all(&decoded) {
                eprintln!("error: failed to write to stdout: {}", e);
                return ExitCode::FAILURE;
            }

            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("error: unknown command '{}'. Use 'encode' or 'decode'.", command);
            return ExitCode::FAILURE;
        }
    }
}
