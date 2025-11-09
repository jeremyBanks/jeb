use clap::Parser;
use color_eyre::eyre::{Context, Result};
use jeb::{apply_sort_buffer, merge_sorted_streams, parse_json_stream, JsonObject, SortSpec};
use serde_json::Value;
use std::fs::{self, File};
use std::io::{self, BufWriter, Read, Write};
use std::path::Path;
use tracing::{debug, info, warn};
use tracing_subscriber;

/// JSON Entity Bucket - Merge, format, and search JSON
#[derive(Parser, Debug)]
#[command(name = "jeb")]
#[command(about = "JSON Entity Bucket - Merge, format, and search JSON")]
#[command(long_about = "JSON Entity Bucket - Merge, format, and search JSON\n\nWith no arguments, reads from stdin and writes to stdout.\nUse '-' to explicitly specify stdin or stdout.\n\nInput arguments starting with { or [ are treated as inline JSON.")]
struct Cli {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Sort buffer size for correcting slight disorder (0 to disable)
    #[arg(short = 'b', long, default_value = "128")]
    buffer_size: usize,

    /// Sort keys in JSON objects (false=unsorted, true=sorted, or JSON array like '["key1", true, "key2"]')
    #[arg(short = 's', long, default_value = "false")]
    sort: String,

    /// Input file (use '-' for stdin). When using positional args, this is also the output file.
    /// Arguments starting with { or [ are treated as inline JSON.
    #[arg(value_name = "FILE")]
    files: Vec<String>,

    /// Input file(s) - alternative to positional arguments (auto-detects inline JSON)
    #[arg(short, long, value_name = "FILE")]
    from: Vec<String>,

    /// Input file path(s) - never treat as inline JSON
    #[arg(long, value_name = "PATH")]
    from_path: Vec<String>,

    /// Input JSON string(s) - always treat as inline JSON
    #[arg(long, value_name = "JSON")]
    from_string: Vec<String>,

    /// Output file (use '-' for stdout) - alternative to positional arguments
    #[arg(short, long, value_name = "FILE")]
    to: Option<String>,
}

/// Represents a source of JSON input
#[derive(Debug, Clone)]
enum InputSource {
    /// Read from a file path or stdin (-)
    FilePath(String),
    /// Parse from inline JSON string
    InlineJson(String),
}

/// Load jeb.json config file from current directory and convert to CLI arguments
fn load_config_args() -> Vec<String> {
    let config_path = Path::new("jeb.json");
    if !config_path.exists() {
        return vec![];
    }

    match std::fs::read_to_string(config_path) {
        Ok(contents) => {
            match serde_json::from_str::<Value>(&contents) {
                Ok(Value::Object(map)) => {
                    let mut args = Vec::new();
                    for (key, value) in map {
                        // Convert each key-value to --key=value format
                        let arg = match value {
                            Value::String(s) => format!("--{}={}", key, s),
                            Value::Number(n) => format!("--{}={}", key, n),
                            Value::Bool(b) => format!("--{}={}", key, b),
                            Value::Array(_) | Value::Object(_) => {
                                // For complex types, serialize back to JSON
                                format!("--{}={}", key, serde_json::to_string(&value).unwrap_or_default())
                            }
                            Value::Null => continue, // Skip null values
                        };
                        args.push(arg);
                    }
                    debug!("Loaded {} config arguments from jeb.json", args.len());
                    args
                }
                Ok(_) => {
                    warn!("jeb.json must contain a JSON object at the top level");
                    vec![]
                }
                Err(e) => {
                    warn!("Failed to parse jeb.json: {}", e);
                    vec![]
                }
            }
        }
        Err(e) => {
            warn!("Failed to read jeb.json: {}", e);
            vec![]
        }
    }
}

fn main() -> Result<()> {
    // Initialize color_eyre for better error reporting
    color_eyre::install()?;

    // Load config file arguments and merge with command-line arguments
    let mut args: Vec<String> = std::env::args().collect();
    let config_args = load_config_args();
    args.extend(config_args);

    let cli = Cli::parse_from(args);

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

    // Parse sort specification
    let sort_spec = SortSpec::parse(&cli.sort)
        .map_err(|e| color_eyre::eyre::eyre!("Invalid sort specification: {}", e))?;

    debug!("Sort spec: {:?}", sort_spec);

    // Validate and determine input/output files
    let (input_sources, output_file) = parse_io_args(&cli)
        .map_err(|e| color_eyre::eyre::eyre!("{}", e))?;

    debug!("Input sources: {} items", input_sources.len());
    debug!("Output file: {:?}", output_file);

    // Process JSON from all input sources, treating each as a sorted stream
    let mut streams = Vec::new();

    for input_source in &input_sources {
        let objects = read_json_from_source(input_source)
            .wrap_err_with(|| format!("Failed to read from source: {:?}", input_source))?;

        let source_desc = match input_source {
            InputSource::FilePath(p) => p.clone(),
            InputSource::InlineJson(_) => "<inline>".to_string(),
        };
        debug!("Read {} objects from {}", objects.len(), source_desc);
        streams.push(objects);
    }

    // Merge all sorted streams into a single sorted output
    let merged_objects = merge_sorted_streams(streams);

    // Apply sort buffer to correct slight disorder
    let all_objects = apply_sort_buffer(merged_objects, cli.buffer_size);
    info!("Total objects after sort buffer: {}", all_objects.len());

    // Apply key sorting to each object
    let all_objects: Vec<JsonObject> = all_objects
        .into_iter()
        .map(|obj| sort_spec.apply(&obj))
        .collect();
    debug!("Applied key sorting to all objects");

    // Determine if we need safe in-place modification
    // Check if output file is used as any input file path
    let input_file_paths: Vec<String> = input_sources
        .iter()
        .filter_map(|src| match src {
            InputSource::FilePath(p) => Some(p.clone()),
            InputSource::InlineJson(_) => None,
        })
        .collect();
    let needs_safe_write = output_file != "-" && input_file_paths.contains(&output_file);

    if needs_safe_write {
        debug!("Output path matches an input path, using safe in-place modification");
        write_json_safely(&output_file, &all_objects)
            .wrap_err_with(|| format!("Failed to write to '{}'", output_file))?;
    } else {
        // Write objects as JSON array to output
        write_json_lines(&output_file, &all_objects)
            .wrap_err_with(|| format!("Failed to write to '{}'", output_file))?;
    }

    info!("JEB completed successfully");
    Ok(())
}

/// Check if a string looks like inline JSON (starts with { or [ and ends with matching brace)
fn is_inline_json(s: &str) -> bool {
    let trimmed = s.trim();
    (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
}

fn parse_io_args(cli: &Cli) -> Result<(Vec<InputSource>, String), String> {
    let has_positional = !cli.files.is_empty();
    let has_to = cli.to.is_some();

    let mut all_inputs: Vec<InputSource> = Vec::new();

    // Collect inputs from --from-path (always file paths)
    for path in &cli.from_path {
        all_inputs.push(InputSource::FilePath(path.clone()));
    }

    // Collect inputs from --from-string (always inline JSON)
    for json in &cli.from_string {
        all_inputs.push(InputSource::InlineJson(json.clone()));
    }

    // Collect inputs from --from (auto-detect)
    for arg in &cli.from {
        if is_inline_json(arg) {
            debug!("Detected inline JSON in --from argument");
            all_inputs.push(InputSource::InlineJson(arg.clone()));
        } else {
            all_inputs.push(InputSource::FilePath(arg.clone()));
        }
    }

    // Collect inputs from positional arguments (auto-detect)
    for arg in &cli.files {
        if is_inline_json(arg) {
            debug!("Detected inline JSON in positional argument");
            all_inputs.push(InputSource::InlineJson(arg.clone()));
        } else {
            all_inputs.push(InputSource::FilePath(arg.clone()));
        }
    }

    // Determine output
    let output = if has_to {
        let out = cli.to.clone().unwrap();
        // Validate output is not inline JSON
        if is_inline_json(&out) {
            return Err("Output path cannot be inline JSON (starts with { or [)".to_string());
        }
        out
    } else if has_positional && !cli.files.is_empty() {
        // First positional argument is output only if it's NOT inline JSON
        let first = &cli.files[0];
        if is_inline_json(first) {
            // All positionals are inputs, output to stdout
            "-".to_string()
        } else {
            // First positional is output file
            first.clone()
        }
    } else {
        // Default to stdout
        "-".to_string()
    };

    // If no inputs specified, default to stdin
    if all_inputs.is_empty() {
        all_inputs.push(InputSource::FilePath("-".to_string()));
    }

    Ok((all_inputs, output))
}

fn read_json_from_source(
    source: &InputSource,
) -> Result<Vec<JsonObject>> {
    match source {
        InputSource::FilePath(path) => read_json_from_file(path),
        InputSource::InlineJson(json) => {
            debug!("Parsing inline JSON");
            parse_json_stream(json).map_err(|e| color_eyre::eyre::eyre!("{}", e))
        }
    }
}

fn read_json_from_file(file_path: &str) -> Result<Vec<JsonObject>> {
    let mut reader: Box<dyn Read> = if file_path == "-" {
        Box::new(io::stdin())
    } else {
        Box::new(File::open(file_path)?)
    };

    let mut content = String::new();
    reader.read_to_string(&mut content)?;

    parse_json_stream(&content).map_err(|e| color_eyre::eyre::eyre!("{}", e))
}

fn write_json_lines(
    file_path: &str,
    objects: &[JsonObject],
) -> Result<()> {
    let mut writer: BufWriter<Box<dyn Write>> = if file_path == "-" {
        BufWriter::new(Box::new(io::stdout()))
    } else {
        BufWriter::new(Box::new(File::create(file_path)?))
    };

    write_json_array(&mut writer, objects)?;
    writer.flush()?;
    Ok(())
}

fn write_json_array<W: Write>(
    writer: &mut W,
    objects: &[JsonObject],
) -> Result<()> {
    // Write as a JSON array, but line-by-line friendly:
    // - First line: [ followed by first object
    // - Middle lines: , followed by each object
    // - Last line: ]
    // This makes it valid JSON but also line-by-line processable (skip first char)

    for (i, obj) in objects.iter().enumerate() {
        let json_str = serde_json::to_string(obj)?;
        if i == 0 {
            writeln!(writer, "[{}", json_str)?;
        } else {
            writeln!(writer, ",{}", json_str)?;
        }
    }

    // Write closing bracket (or just [] if empty)
    if objects.is_empty() {
        writeln!(writer, "[]")?;
    } else {
        writeln!(writer, "]")?;
    }

    Ok(())
}

fn write_json_safely(
    file_path: &str,
    objects: &[JsonObject],
) -> Result<()> {
    let path = Path::new(file_path);
    let parent = path.parent().unwrap_or_else(|| Path::new("."));

    // Create temp file in the same directory
    let temp_path = parent.join(format!(".jeb-tmp-{}", std::process::id()));
    let backup_path = parent.join(format!("{}.bak-{}",
        path.file_name().unwrap_or_default().to_string_lossy(),
        std::process::id()
    ));

    debug!("Writing to temp file: {:?}", temp_path);

    // Write to temp file
    {
        let temp_file = File::create(&temp_path)?;
        let mut writer = BufWriter::new(temp_file);
        write_json_array(&mut writer, objects)?;
        writer.flush()?;
    }

    // Rename original to backup
    if path.exists() {
        debug!("Renaming original {:?} to backup {:?}", path, backup_path);
        fs::rename(path, &backup_path)?;
    }

    // Rename temp to target
    debug!("Renaming temp {:?} to target {:?}", temp_path, path);
    fs::rename(&temp_path, path)?;

    // Delete backup
    if backup_path.exists() {
        debug!("Deleting backup {:?}", backup_path);
        if let Err(e) = fs::remove_file(&backup_path) {
            warn!("Failed to delete backup file {:?}: {}", backup_path, e);
        }
    }

    info!("Safely wrote to {:?}", path);
    Ok(())
}
