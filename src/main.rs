use clap::Parser;
use jeb::{merge_sorted_streams, parse_json_stream, JsonObject};
use std::fs::File;
use std::io::{self, BufWriter, Read, Write};
use tracing::{debug, error, info};
use tracing_subscriber;

/// JSON Entity Bucket - Merge, format, and search JSON
#[derive(Parser, Debug)]
#[command(name = "jeb")]
#[command(about = "JSON Entity Bucket - Merge, format, and search JSON")]
#[command(long_about = "JSON Entity Bucket - Merge, format, and search JSON\n\nWith no arguments, reads from stdin and writes to stdout.\nUse '-' to explicitly specify stdin or stdout.")]
struct Cli {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Input file (use '-' for stdin). When using positional args, this is also the output file.
    #[arg(value_name = "FILE")]
    files: Vec<String>,

    /// Input file(s) - alternative to positional arguments
    #[arg(short, long, value_name = "FILE")]
    from: Vec<String>,

    /// Output file (use '-' for stdout) - alternative to positional arguments
    #[arg(short, long, value_name = "FILE")]
    to: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    // Initialize tracing with env-based configuration
    // Defaults to "info" level, can be overridden with RUST_LOG env var
    // Always writes to stderr
    let default_filter = if cli.debug { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(default_filter)),
        )
        .init();

    info!("JEB starting...");

    // Validate and determine input/output files
    let (input_files, output_file) = match parse_io_args(&cli) {
        Ok((inputs, output)) => (inputs, output),
        Err(e) => {
            error!("{}", e);
            std::process::exit(1);
        }
    };

    debug!("Input files: {:?}", input_files);
    debug!("Output file: {:?}", output_file);

    // Process JSON from all input files, treating each as a sorted stream
    let mut streams = Vec::new();

    for input_file in &input_files {
        match read_json_objects(input_file) {
            Ok(objects) => {
                debug!("Read {} objects from {}", objects.len(), input_file);
                streams.push(objects);
            }
            Err(e) => {
                error!("Failed to read from '{}': {}", input_file, e);
                std::process::exit(1);
            }
        }
    }

    // Merge all sorted streams into a single sorted output
    let all_objects = merge_sorted_streams(streams);
    info!("Total objects after merge: {}", all_objects.len());

    // Write objects as JSON lines to output
    if let Err(e) = write_json_lines(&output_file, &all_objects) {
        error!("Failed to write to '{}': {}", output_file, e);
        std::process::exit(1);
    }

    info!("JEB completed successfully");
}

fn parse_io_args(cli: &Cli) -> Result<(Vec<String>, String), String> {
    let has_positional = !cli.files.is_empty();
    let has_from = !cli.from.is_empty();
    let has_to = cli.to.is_some();

    // Error if both positional and both --from/--to are present
    if has_positional && has_from && has_to {
        return Err(
            "Cannot specify both positional arguments and --from/--to options".to_string(),
        );
    }

    // Use --from/--to if present
    if has_from || has_to {
        let inputs = if has_from {
            cli.from.clone()
        } else {
            vec!["-".to_string()] // Default to stdin
        };
        let output = cli.to.clone().unwrap_or_else(|| "-".to_string());
        return Ok((inputs, output));
    }

    // Use positional arguments
    if has_positional {
        if cli.files.len() == 1 {
            // Single file is both input and output
            let file = cli.files[0].clone();
            return Ok((vec![file.clone()], file));
        } else {
            // First file is input/output, rest are additional inputs
            let output = cli.files[0].clone();
            let inputs = cli.files.clone();
            return Ok((inputs, output));
        }
    }

    // Default: stdin to stdout
    Ok((vec!["-".to_string()], "-".to_string()))
}

fn read_json_objects(file_path: &str) -> Result<Vec<JsonObject>, Box<dyn std::error::Error>> {
    let mut reader: Box<dyn Read> = if file_path == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(File::open(file_path)?)
    };

    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    parse_json_stream(&content)
}

fn write_json_lines(
    file_path: &str,
    objects: &[JsonObject],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut writer: BufWriter<Box<dyn Write>> = if file_path == "-" {
        BufWriter::new(Box::new(io::stdout()))
    } else {
        BufWriter::new(Box::new(File::create(file_path)?))
    };

    for obj in objects {
        let json_str = serde_json::to_string(obj)?;
        writeln!(writer, "{}", json_str)?;
    }

    writer.flush()?;
    Ok(())
}
