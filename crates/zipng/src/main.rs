use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::path::Path;
use std::process::ExitCode;

use indexmap::IndexMap;
use rgb::RGB8;
use zipng::Files;

const MAX_FILE_SIZE: usize = 60 * 1024; // 60KB limit

fn print_help() {
    eprintln!(
        "Usage: zipng -o <output.png> [options] <files...>

Creates a polyglot PNG+ZIP file from input files.

Options:
  -o, --output <file>  Output file path (required)
  -j                   Junk paths - store just filenames, not full paths
  -@                   Read file list from stdin (one per line)
  -q                   Quiet mode - no output on success
  --colors <hex,...>   Custom palette from comma-separated hex RGB colors
                       (e.g. --colors ffffff,ff0000,000000)
  -h, --help           Show this help

Examples:
  zipng -o archive.png file1.txt file2.txt
  zipng -j -o archive.png path/to/file.txt
  zipng --colors ffffff,3b82f6,000000 -o archive.png file.txt
  find . -name '*.rs' | zipng -@ -o archive.png"
    );
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        print_help();
        return ExitCode::from(1);
    }

    let mut output_path: Option<String> = None;
    let mut input_files: Vec<String> = Vec::new();
    let mut junk_paths = false;
    let mut read_stdin = false;
    let mut quiet = false;
    let mut custom_colors: Option<Vec<RGB8>> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-h" | "--help" => {
                print_help();
                return ExitCode::SUCCESS;
            }
            "-o" | "--output" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: -o requires an argument");
                    return ExitCode::from(1);
                }
                output_path = Some(args[i + 1].clone());
                i += 2;
            }
            "-j" => {
                junk_paths = true;
                i += 1;
            }
            "-@" => {
                read_stdin = true;
                i += 1;
            }
            "-q" => {
                quiet = true;
                i += 1;
            }
            "--colors" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --colors requires an argument");
                    return ExitCode::from(1);
                }
                match parse_colors(&args[i + 1]) {
                    Ok(colors) => custom_colors = Some(colors),
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return ExitCode::from(1);
                    }
                }
                i += 2;
            }
            arg if arg.starts_with('-') => {
                eprintln!("Error: unknown option: {}", arg);
                return ExitCode::from(1);
            }
            _ => {
                input_files.push(args[i].clone());
                i += 1;
            }
        }
    }

    // Read additional files from stdin if -@ specified
    if read_stdin {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            match line {
                Ok(path) => {
                    let path = path.trim();
                    if !path.is_empty() {
                        input_files.push(path.to_string());
                    }
                }
                Err(e) => {
                    eprintln!("Error reading stdin: {}", e);
                    return ExitCode::from(1);
                }
            }
        }
    }

    // Validate required options
    let output_path = match output_path {
        Some(p) => p,
        None => {
            eprintln!("Error: -o/--output is required");
            return ExitCode::from(1);
        }
    };

    if input_files.is_empty() {
        eprintln!("Error: no input files specified");
        return ExitCode::from(1);
    }

    // Build file map
    let mut files: IndexMap<Vec<u8>, Vec<u8>> = IndexMap::new();

    for path in &input_files {
        let file_path = Path::new(path);

        // Check file exists
        if !file_path.exists() {
            eprintln!("Error: file not found: {}", path);
            return ExitCode::from(1);
        }

        // Read file contents
        let contents = match fs::read(file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Error reading {}: {}", path, e);
                return ExitCode::from(1);
            }
        };

        // Check size limit
        if contents.len() > MAX_FILE_SIZE {
            eprintln!(
                "Error: {} exceeds 60KB limit ({:.1} KB)",
                path,
                contents.len() as f64 / 1024.0
            );
            return ExitCode::from(1);
        }

        // Determine archive path
        let archive_path = if junk_paths {
            file_path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| path.clone())
        } else {
            path.clone()
        };

        files.insert(archive_path.into_bytes(), contents);
    }

    // Generate polyglot PNG+ZIP
    let custom_palette = custom_colors
        .map(|colors| zipng::palettes::perceptual::generate(&colors));
    let output = zipng::zipng_with_palette(&Files::from(files), custom_palette.as_deref());

    // Write output
    if let Err(e) = fs::write(&output_path, &output) {
        eprintln!("Error writing {}: {}", output_path, e);
        return ExitCode::from(1);
    }

    // Print summary
    if !quiet {
        let size_kib = output.len() as f64 / 1024.0;
        let file_count = input_files.len();
        let file_word = if file_count == 1 { "file" } else { "files" };
        println!(
            "Created {} ({} {}, {:.1} KiB)",
            output_path, file_count, file_word, size_kib
        );
    }

    ExitCode::SUCCESS
}

fn parse_colors(s: &str) -> Result<Vec<RGB8>, String> {
    let colors: Result<Vec<RGB8>, String> = s
        .split(',')
        .map(|hex| {
            let hex = hex.trim().trim_start_matches('#');
            if hex.len() != 6 {
                return Err(format!("invalid hex color '{}': expected 6 hex digits", hex));
            }
            let r = u8::from_str_radix(&hex[0..2], 16)
                .map_err(|_| format!("invalid hex color '{}'", hex))?;
            let g = u8::from_str_radix(&hex[2..4], 16)
                .map_err(|_| format!("invalid hex color '{}'", hex))?;
            let b = u8::from_str_radix(&hex[4..6], 16)
                .map_err(|_| format!("invalid hex color '{}'", hex))?;
            Ok(RGB8::new(r, g, b))
        })
        .collect();
    let colors = colors?;
    if colors.is_empty() {
        return Err("--colors requires at least one color".to_string());
    }
    Ok(colors)
}
