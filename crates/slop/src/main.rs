use {
    clap::Parser,
    color_eyre::eyre::{Context, Result},
    futures::stream::{self, StreamExt},
    slop::{JsonObject, SortSpec, apply_sort_buffer, merge_sorted_streams, parse_json_stream},
    serde_json::Value,
    std::path::Path,
    tokio::{
        fs::File,
        io::{AsyncWriteExt, BufWriter, stdin, stdout},
    },
    tracing::{debug, info, warn},
};

/// JSON Entity Bucket - Merge, format, and search JSON
#[derive(Parser, Debug)]
#[command(name = "jeb")]
#[command(about = "JSON Entity Bucket - Merge, format, and search JSON")]
#[command(
    long_about = "JSON Entity Bucket - Merge, format, and search JSON\n\nWith no arguments, reads \
                  from stdin and writes to stdout.\nUse '-' to explicitly specify stdin or \
                  stdout.\n\nInput arguments starting with { or [ are treated as inline JSON."
)]
struct Cli {
    /// Enable debug logging
    #[arg(short, long)]
    debug: bool,

    /// Sort buffer size for correcting slight disorder (0 to disable)
    #[arg(short = 'b', long, default_value = "128")]
    buffer_size: usize,

    /// Sort keys in JSON objects (false=unsorted, true=sorted, or JSON array
    /// like '["key1", true, "key2"]')
    #[arg(short = 's', long, default_value = "false")]
    sort: String,

    /// Input file (use '-' for stdin). When using positional args, this is also
    /// the output file. Arguments starting with { or [ are treated as
    /// inline JSON.
    #[arg(value_name = "FILE")]
    files: Vec<String>,

    /// Input file(s) - alternative to positional arguments (auto-detects inline
    /// JSON)
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

/// Load jeb.json config file from current directory and convert to CLI
/// arguments
fn load_config_args() -> Vec<String> {
    let config_path = Path::new("jeb.json");
    if !config_path.exists() {
        return vec![];
    }

    match std::fs::read_to_string(config_path) {
        Ok(contents) => match serde_json::from_str::<Value>(&contents) {
            Ok(Value::Object(map)) => {
                let mut args = Vec::new();
                for (key, value) in map {
                    // Convert each key-value to --key=value format
                    let arg = match value {
                        Value::String(s) => format!("--{key}={s}"),
                        Value::Number(n) => format!("--{key}={n}"),
                        Value::Bool(b) => format!("--{key}={b}"),
                        Value::Array(_) | Value::Object(_) => {
                            // For complex types, serialize back to JSON
                            format!(
                                "--{}={}",
                                key,
                                serde_json::to_string(&value).unwrap_or_default()
                            )
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
        },
        Err(e) => {
            warn!("Failed to read jeb.json: {}", e);
            vec![]
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
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
    let (input_sources, output_file) =
        parse_io_args(&cli).map_err(|e| color_eyre::eyre::eyre!("{}", e))?;

    debug!("Input sources: {} items", input_sources.len());
    debug!("Output file: {:?}", output_file);

    // Process JSON from all input sources as async streams
    let mut streams = Vec::new();

    for input_source in &input_sources {
        let stream = create_stream_from_source(input_source.clone()).await?;
        streams.push(stream);
    }

    // Merge all sorted streams into a single sorted output
    let merged = Box::pin(merge_sorted_streams(streams));

    // Apply sort buffer to correct slight disorder
    let buffered = Box::pin(apply_sort_buffer(merged, cli.buffer_size));

    // Apply key sorting to each object
    let sorted_keys = buffered.map(move |obj| sort_spec.apply(&obj));

    // Collect all objects
    let all_objects: Vec<JsonObject> = sorted_keys.collect().await;
    info!("Total objects after processing: {}", all_objects.len());

    // Determine if we need safe in-place modification
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
        write_json_safely(&output_file, &all_objects).await?;
    } else {
        // Write objects as JSON array to output
        write_json_lines(&output_file, &all_objects).await?;
    }

    info!("JEB completed successfully");
    Ok(())
}

/// Create a stream from an input source
async fn create_stream_from_source(
    source: InputSource,
) -> Result<
    std::pin::Pin<
        Box<dyn futures::Stream<Item = Result<JsonObject, slop::JsonError>> + Send + 'static>,
    >,
> {
    match source {
        InputSource::FilePath(path) if path == "-" => {
            debug!("Reading from stdin");
            Ok(Box::pin(parse_json_stream(tokio::io::BufReader::new(
                stdin(),
            ))))
        }
        InputSource::FilePath(path) => {
            debug!("Reading from file: {}", path);
            let file = File::open(&path)
                .await
                .wrap_err_with(|| format!("Failed to open file: {path}"))?;
            Ok(Box::pin(parse_json_stream(tokio::io::BufReader::new(file))))
        }
        InputSource::InlineJson(json) => {
            debug!("Parsing inline JSON");
            // Parse inline JSON synchronously and convert to stream
            match slop::parse_json_string(&json) {
                Ok(objects) => Ok(Box::pin(stream::iter(objects.into_iter().map(Ok)))),
                Err(e) => Err(color_eyre::eyre::eyre!(
                    "Failed to parse inline JSON: {}",
                    e
                )),
            }
        }
    }
}

/// Check if a string looks like inline JSON (starts with { or [ and ends with
/// matching brace)
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

async fn write_json_lines(file_path: &str, objects: &[JsonObject]) -> Result<()> {
    let mut writer: BufWriter<Box<dyn tokio::io::AsyncWrite + Unpin>> = if file_path == "-" {
        BufWriter::new(Box::new(stdout()))
    } else {
        BufWriter::new(Box::new(File::create(file_path).await?))
    };

    write_json_array(&mut writer, objects).await?;
    writer.flush().await?;
    Ok(())
}

async fn write_json_array<W: tokio::io::AsyncWrite + Unpin>(
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
            writer
                .write_all(format!("[{json_str}\n").as_bytes())
                .await?;
        } else {
            writer
                .write_all(format!(",{json_str}\n").as_bytes())
                .await?;
        }
    }

    // Write closing bracket (or just [] if empty)
    if objects.is_empty() {
        writer.write_all(b"[]\n").await?;
    } else {
        writer.write_all(b"]\n").await?;
    }

    Ok(())
}

async fn write_json_safely(file_path: &str, objects: &[JsonObject]) -> Result<()> {
    let path = Path::new(file_path);
    let parent = path.parent().unwrap_or_else(|| Path::new("."));

    // Create temp file in the same directory with timestamp for uniqueness
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let temp_path = parent.join(format!(".jeb-tmp-{}-{}", std::process::id(), timestamp));
    let backup_path = parent.join(format!(
        "{}.bak-{}-{}",
        path.file_name().unwrap_or_default().to_string_lossy(),
        std::process::id(),
        timestamp
    ));

    debug!("Writing to temp file: {:?}", temp_path);

    // Write to temp file
    {
        let temp_file = File::create(&temp_path).await?;
        let mut writer = BufWriter::new(temp_file);
        write_json_array(&mut writer, objects).await?;
        writer.flush().await?;
    }

    // Rename original to backup
    if path.exists() {
        debug!("Renaming original {:?} to backup {:?}", path, backup_path);
        tokio::fs::rename(path, &backup_path).await?;
    }

    // Rename temp to target
    debug!("Renaming temp {:?} to target {:?}", temp_path, path);
    tokio::fs::rename(&temp_path, path).await?;

    // Delete backup
    if backup_path.exists() {
        debug!("Deleting backup {:?}", backup_path);
        if let Err(e) = tokio::fs::remove_file(&backup_path).await {
            warn!("Failed to delete backup file {:?}: {}", backup_path, e);
        }
    }

    info!("Safely wrote to {:?}", path);
    Ok(())
}
