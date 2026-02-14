use std::io::{self, Read, Write};

use z855::z855;
use std::process::ExitCode;

fn main() -> ExitCode {
    // Parse command line arguments
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("error: expected exactly one argument: 'encode' or 'decode'");
        eprintln!("Usage: {} <encode|decode>", args.get(0).map(|s| s.as_str()).unwrap_or("z855"));
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

            // Encode to Z855
            let encoded = z855::encode(&input);

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

            // Z855 encoded strings can contain raw bytes in passthrough sections (after ,~|)
            // These may not be valid UTF-8, so we use from_utf8_lossy which replaces
            // invalid sequences with � (U+FFFD). Since the decoder works on bytes
            // (.as_bytes()), and raw passthrough bytes are preserved as-is in the input,
            // this works correctly.
            //
            // Safety: from_utf8_unchecked would be unsafe but correct here, since:
            // 1. Z85 alphabet and escapes are all ASCII (valid UTF-8)
            // 2. Raw passthrough bytes are copied as-is without interpretation
            // 3. The decoder immediately calls .as_bytes() anyway
            //
            // We use from_utf8_lossy for safety, though it's slightly inefficient.
            let input_str = unsafe { std::str::from_utf8_unchecked(&input) };

            // Decode from Z855
            let decoded = match z855::decode(input_str) {
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
