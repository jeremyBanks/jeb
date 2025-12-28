#![cfg(feature = "bin")]

use {
    jeb::{
        Panic,
        model::Bytes,
    },
    jeb_common::shell_tokenizer,
    jeb_streaming,
    owo_colors::OwoColorize,
    regex::Regex,
    std::{
        convert::Infallible,
        io::{
            Read,
            Write,
        },
        mem::take,
        sync::LazyLock,
    },
    tracing::debug,
};

/// Pre-defined aliases that expand a single command into one or more commands.
static ALIASES: &[(&str, &[&str])] = &[("to-jeb85-lines", &[
    "encode-jeb85",
    "split-80",
    "join-lines",
])];

static PRELUDE: &str = include_str!("jeb/prelude.jeb");

/// Expand an alias into its component commands, or return the original command.
fn expand_alias(command: &str) -> Vec<String> {
    for (alias, expansion) in ALIASES {
        if command == *alias {
            return expansion.iter().map(|s| s.to_string()).collect();
        }
    }
    vec![command.to_string()]
}

// XXX: Consider switching to a real entry point so we can do
//      set .unhandled_panic(UnhandledPanic::ShutdownRuntime).
#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), Infallible> {
    inner_main().await.ok();
    Ok(())
}

pub async fn inner_main() -> Result<(), Panic> {
    color_eyre::install()?;
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .pretty()
        .init();

    let mut args = Vec::<String>::from_iter(std::env::args());
    let own_path: String = args.remove(0);

    // Parse prelude and prepend to args
    let prelude_result = shell_tokenizer::tokenize(PRELUDE.as_bytes());
    for error in &prelude_result.errors {
        debug!("prelude error: {error}");
    }
    let prelude_args: Vec<String> = prelude_result
        .args
        .into_iter()
        .map(|bytes| String::from_utf8(bytes).expect("prelude should be valid UTF-8"))
        .collect();

    args = prelude_args
        .into_iter()
        .chain(args)
        .flat_map(|s| {
            static REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r#"\s\|\s"#).unwrap());

            if REGEX.is_match(&s) {
                s.split('|')
                    .map(|s| s.trim_ascii().to_string())
                    .collect::<Vec<String>>()
            } else {
                vec![s]
            }
        })
        // Expand any aliases into their component commands
        .flat_map(|s| expand_alias(&s))
        .collect();

    let mut commands = args;
    let mut commands_fmt = commands
        .iter()
        .map(|s| s.yellow().to_string())
        .collect::<Vec<String>>()
        .join(" ");

    if commands.is_empty() {
        commands.push("help".to_string());
        commands_fmt.push_str("help".red().to_string().as_str());
    }

    if commands.last().map(|s| s.as_str()) != Some("stdout") {
        commands.push("stdout".to_string());
        commands_fmt.push_str(" stdout".red().to_string().as_str());
    }

    let mut state = Vec::<Bytes>::new();

    eprintln!("{} {}", own_path.magenta(), commands_fmt);

    let mut _default_mode: &'static str = "last";

    for command in commands {
        state = match command.as_str() {
            "help" | "--help" | "-h" | "-?" => help(state)?,
            "stdin" => stdin(state)?,
            "stdout" => stdout(state)?,
            "self" => self_(state)?,
            "first" => first(state)?,
            "last" => last(state)?,
            "split-lines" => split_lines(state).await?,
            "split-shell" => split_shell(state).await?,
            "join" => join(state)?,
            "join-lines" => join_lines(state)?,
            "join-space" => join_space(state)?,
            "collapse" => collapse(state).await?,
            "filter" => filter(state).await?,
            "encode-z85" => encode_z85(state)?,
            "decode-z85" => decode_z85(state)?,
            "encode-jeb85" => encode_jeb85(state)?,
            "parse-hex" => parse_hex(state).await?,
            "to-hex" => to_hex(state).await?,
            "parse-binary" => parse_binary(state).await?,
            "to-binary" => to_binary(state).await?,
            "split-whitespace" => split_whitespace(state).await?,
            "--all" => {
                _default_mode = "all";
                state
            }
            "--last" => {
                _default_mode = "last";
                state
            }
            "--first" => {
                _default_mode = "first";
                state
            }
            _ if command.contains('=') => {
                // Assignment - no-op for now
                state
            }
            _ => {
                if command.starts_with(".") || command.starts_with("/") {
                    read(state, &command)?
                } else if let Some(arg) = command.strip_prefix("last-") {
                    last_n(state, arg)?
                } else if let Some(arg) = command.strip_prefix("first-") {
                    first_n(state, arg)?
                } else if let Some(arg) = command.strip_prefix("split-") {
                    split_n(state, arg).await?
                } else if let Some(arg) = command.strip_prefix("find-") {
                    find_target(state, arg)?
                } else {
                    eprintln!("error: unrecognized argument: {command}");
                    std::process::exit(1);
                }
            }
        }
    }

    Ok(())
}

fn help(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    static README: &str = include_str!("../../README.md");
    state.push(README.into());

    Ok(state)
}

fn read(mut state: Vec<Bytes>, path: &str) -> Result<Vec<Bytes>, Panic> {
    let data = std::fs::read(path)?;
    state.push(Bytes::from(data));
    Ok(state)
}

fn self_(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    let own_path = std::env::current_exe()?;
    let own_data = std::fs::read(own_path)?;
    state.push(Bytes::from(own_data));
    Ok(state)
}

fn stdin(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    let mut buffer = Vec::<u8>::new();
    std::io::stdin().read_to_end(&mut buffer)?;
    state.push(Bytes::from(buffer));
    Ok(state)
}

fn stdout(state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    for item in state {
        std::io::stdout().write_all(&item)?;
    }
    Ok(Vec::new())
}

fn encode_z85(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    for piece in &mut state {
        let bytes = take(piece);
        let encoded = jeb::z85::encode_z85(&bytes);
        *piece = encoded.into();
    }
    Ok(state)
}

fn encode_jeb85(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    for piece in &mut state {
        let bytes = take(piece);
        let encoded = jeb::jeb85::encode_jeb85(&bytes);
        *piece = encoded.into();
    }
    Ok(state)
}

fn decode_z85(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    for piece in &mut state {
        let encoded = take(piece);
        let decoded = jeb::z85::decode_z85(&encoded)?;
        *piece = decoded.into();
    }
    Ok(state)
}

fn first(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    while state.len() > 1 {
        state.pop();
    }
    Ok(state)
}

fn first_n(mut state: Vec<Bytes>, arg: &str) -> Result<Vec<Bytes>, Panic> {
    let n: usize = arg.parse()?;
    while state.len() > n {
        state.pop();
    }
    Ok(state)
}

fn last(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    let last = state.pop();
    state.clear();
    if let Some(item) = last {
        state.push(item);
    }
    Ok(state)
}

fn last_n(mut state: Vec<Bytes>, arg: &str) -> Result<Vec<Bytes>, Panic> {
    let n: usize = arg.parse()?;
    if state.len() > n {
        state.drain(0..state.len() - n);
    }
    Ok(state)
}

async fn collapse(state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    use futures::StreamExt;

    // Convert Vec<Bytes> to stream of Vec<u8>
    let byte_vecs: Vec<Vec<u8>> = state.into_iter().map(|b| b.to_vec()).collect();
    let source = jeb_streaming::bytes_source(byte_vecs);

    // Apply transformation
    let transformed = jeb_streaming::collapse(source);

    // Collect back to Vec<Bytes>
    let items: Vec<_> = transformed.collect().await;
    let mut result = Vec::new();

    for item_result in items {
        match item_result {
            Ok(jeb_streaming::Item::Bytes(bytes)) => {
                result.push(Bytes::from(bytes.to_vec()));
            }
            Ok(jeb_streaming::Item::Text(text)) => {
                result.push(Bytes::from(text.as_bytes().to_vec()));
            }
            Ok(_) => {} // Skip other item types
            Err(e) => return Err(e.into()),
        }
    }
    Ok(result)
}

async fn split_lines(state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    use futures::StreamExt;

    // Convert Vec<Bytes> to stream of Vec<u8>
    let byte_vecs: Vec<Vec<u8>> = state.into_iter().map(|b| b.to_vec()).collect();
    let source = jeb_streaming::bytes_source(byte_vecs);

    // Apply transformation (lines() splits on newlines)
    let transformed = jeb_streaming::lines(source);

    // Collect back to Vec<Bytes>
    let items: Vec<_> = transformed.collect().await;
    let mut result = Vec::new();

    for item_result in items {
        match item_result {
            Ok(jeb_streaming::Item::Bytes(bytes)) => {
                result.push(Bytes::from(bytes.to_vec()));
            }
            Ok(jeb_streaming::Item::Text(text)) => {
                result.push(Bytes::from(text.as_bytes().to_vec()));
            }
            Ok(_) => {} // Skip other item types
            Err(e) => return Err(e.into()),
        }
    }
    Ok(result)
}

fn parse_size_notation(arg: &str) -> Result<usize, Panic> {
    let rest = arg.to_ascii_uppercase();
    let mut rest = rest.as_str();

    let mut unit = 1;

    let binary;
    if let Some(next) = rest.strip_suffix("IB") {
        rest = next;
        binary = true;
    } else if let Some(next) = rest.strip_suffix("B") {
        rest = next;
        binary = false;
    } else if let Some(next) = rest.strip_suffix("I") {
        rest = next;
        binary = true;
    } else {
        binary = true;
    }

    loop {
        if let Some(next) = rest.strip_suffix("K") {
            if binary {
                unit <<= 10;
            } else {
                unit *= 1_000;
            }
            rest = next;
        } else if let Some(next) = rest.strip_suffix("M") {
            if binary {
                unit <<= 20;
            } else {
                unit *= 1_000_000;
            }
            rest = next;
        } else if let Some(next) = rest.strip_suffix("G") {
            if binary {
                unit <<= 30;
            } else {
                unit *= 1_000_000_000;
            }
            rest = next;
        } else {
            break;
        }
    }

    let coefficient: usize = rest.parse()?;
    let size = coefficient * unit;
    Ok(size)
}

async fn split_n(state: Vec<Bytes>, arg: &str) -> Result<Vec<Bytes>, Panic> {
    use futures::StreamExt;

    let size = parse_size_notation(arg)?;

    // Convert Vec<Bytes> to stream of Vec<u8>
    let byte_vecs: Vec<Vec<u8>> = state.into_iter().map(|b| b.to_vec()).collect();
    let source = jeb_streaming::bytes_source(byte_vecs);

    // Apply transformation
    let transformed = jeb_streaming::chunks(source, size);

    // Collect back to Vec<Bytes>
    let items: Vec<_> = transformed.collect().await;
    let mut result = Vec::new();

    for item_result in items {
        match item_result {
            Ok(jeb_streaming::Item::Bytes(bytes)) => {
                result.push(Bytes::from(bytes.to_vec()));
            }
            Ok(jeb_streaming::Item::Text(text)) => {
                result.push(Bytes::from(text.as_bytes().to_vec()));
            }
            Ok(_) => {}
            Err(e) => return Err(e.into()),
        }
    }
    Ok(result)
}

fn find_target(state: Vec<Bytes>, arg: &str) -> Result<Vec<Bytes>, Panic> {
    let target = arg.as_bytes();
    let mut result = Vec::<Bytes>::new();
    for bytes in state {
        if bytes.windows(target.len()).any(|window| window == target) {
            result.push(bytes);
        }
    }
    Ok(result)
}

fn join(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    let input = take(&mut state).into_iter().flatten().collect::<Vec<u8>>();
    state.push(Bytes::from(input));
    Ok(state)
}

fn join_lines(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    let input = take(&mut state)
        .into_iter()
        .flat_map(|b| b.into_iter().chain(core::iter::once(b'\n')))
        .collect::<Vec<u8>>();
    state.push(Bytes::from(input));
    Ok(state)
}

fn join_space(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    let mut input = take(&mut state)
        .into_iter()
        .flat_map(|b| b.into_iter().chain(core::iter::once(b' ')))
        .collect::<Vec<u8>>();
    input.pop();
    state.push(Bytes::from(input));
    Ok(state)
}

async fn filter(state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    use futures::StreamExt;

    // Convert Vec<Bytes> to stream of Vec<u8>
    let byte_vecs: Vec<Vec<u8>> = state.into_iter().map(|b| b.to_vec()).collect();
    let source = jeb_streaming::bytes_source(byte_vecs);

    // Apply transformation
    let transformed = jeb_streaming::filter(source);

    // Collect back to Vec<Bytes>
    let items: Vec<_> = transformed.collect().await;
    let mut result = Vec::new();

    for item_result in items {
        match item_result {
            Ok(jeb_streaming::Item::Bytes(bytes)) => {
                result.push(Bytes::from(bytes.to_vec()));
            }
            Ok(jeb_streaming::Item::Text(text)) => {
                result.push(Bytes::from(text.as_bytes().to_vec()));
            }
            Ok(_) => {} // Skip other item types
            Err(e) => return Err(e.into()),
        }
    }
    Ok(result)
}

async fn split_shell(state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    use futures::StreamExt;

    // Convert Vec<Bytes> to stream of Vec<u8>
    let byte_vecs: Vec<Vec<u8>> = state.into_iter().map(|b| b.to_vec()).collect();
    let source = jeb_streaming::bytes_source(byte_vecs);

    // Apply transformation (split_shell tokenizes using shell rules)
    let transformed = jeb_streaming::split_shell(source);

    // Collect back to Vec<Bytes>
    let items: Vec<_> = transformed.collect().await;
    let mut result = Vec::new();

    for item_result in items {
        match item_result {
            Ok(jeb_streaming::Item::Bytes(bytes)) => {
                result.push(Bytes::from(bytes.to_vec()));
            }
            Ok(jeb_streaming::Item::Text(text)) => {
                result.push(Bytes::from(text.as_bytes().to_vec()));
            }
            Ok(_) => {} // Skip other item types
            Err(e) => return Err(e.into()),
        }
    }
    Ok(result)
}

async fn split_whitespace(state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    use futures::StreamExt;

    // Convert Vec<Bytes> to stream of Vec<u8>
    let byte_vecs: Vec<Vec<u8>> = state.into_iter().map(|b| b.to_vec()).collect();
    let source = jeb_streaming::bytes_source(byte_vecs);

    // Apply transformation
    let transformed = jeb_streaming::split_whitespace(source);

    // Collect back to Vec<Bytes>
    let items: Vec<_> = transformed.collect().await;
    let mut result = Vec::new();

    for item_result in items {
        match item_result {
            Ok(jeb_streaming::Item::Bytes(bytes)) => {
                result.push(Bytes::from(bytes.to_vec()));
            }
            Ok(jeb_streaming::Item::Text(text)) => {
                result.push(Bytes::from(text.as_bytes().to_vec()));
            }
            Ok(_) => {} // Skip other item types
            Err(e) => return Err(e.into()),
        }
    }
    Ok(result)
}

async fn parse_hex(state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    use futures::StreamExt;

    // Convert Vec<Bytes> to stream of Vec<u8>
    let byte_vecs: Vec<Vec<u8>> = state.into_iter().map(|b| b.to_vec()).collect();
    let source = jeb_streaming::bytes_source(byte_vecs);

    // Apply transformation
    let transformed = jeb_streaming::parse_hex(source);

    // Collect back to Vec<Bytes>
    let items: Vec<_> = transformed.collect().await;
    let mut result = Vec::new();

    for item_result in items {
        match item_result {
            Ok(jeb_streaming::Item::Bytes(bytes)) => {
                result.push(Bytes::from(bytes.to_vec()));
            }
            Ok(jeb_streaming::Item::Text(text)) => {
                result.push(Bytes::from(text.as_bytes().to_vec()));
            }
            Ok(_) => {} // Skip other item types
            Err(e) => return Err(e.into()),
        }
    }
    Ok(result)
}

async fn to_hex(state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    use futures::StreamExt;

    // Convert Vec<Bytes> to stream of Vec<u8>
    let byte_vecs: Vec<Vec<u8>> = state.into_iter().map(|b| b.to_vec()).collect();
    let source = jeb_streaming::bytes_source(byte_vecs);

    // Apply transformation
    let transformed = jeb_streaming::to_hex(source);

    // Collect back to Vec<Bytes>
    let items: Vec<_> = transformed.collect().await;
    let mut result = Vec::new();

    for item_result in items {
        match item_result {
            Ok(jeb_streaming::Item::Text(text)) => {
                result.push(Bytes::from(text.as_bytes().to_vec()));
            }
            Ok(jeb_streaming::Item::Bytes(bytes)) => {
                result.push(Bytes::from(bytes.to_vec()));
            }
            Ok(_) => {} // Skip other item types
            Err(e) => return Err(e.into()),
        }
    }
    Ok(result)
}

async fn parse_binary(state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    use futures::StreamExt;

    // Convert Vec<Bytes> to stream of Vec<u8>
    let byte_vecs: Vec<Vec<u8>> = state.into_iter().map(|b| b.to_vec()).collect();
    let source = jeb_streaming::bytes_source(byte_vecs);

    // Apply transformation
    let transformed = jeb_streaming::parse_binary(source);

    // Collect back to Vec<Bytes>
    let items: Vec<_> = transformed.collect().await;
    let mut result = Vec::new();

    for item_result in items {
        match item_result {
            Ok(jeb_streaming::Item::Bytes(bytes)) => {
                result.push(Bytes::from(bytes.to_vec()));
            }
            Ok(jeb_streaming::Item::Text(text)) => {
                result.push(Bytes::from(text.as_bytes().to_vec()));
            }
            Ok(_) => {} // Skip other item types
            Err(e) => return Err(e.into()),
        }
    }
    Ok(result)
}

async fn to_binary(state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    use futures::StreamExt;

    // Convert Vec<Bytes> to stream of Vec<u8>
    let byte_vecs: Vec<Vec<u8>> = state.into_iter().map(|b| b.to_vec()).collect();
    let source = jeb_streaming::bytes_source(byte_vecs);

    // Apply transformation
    let transformed = jeb_streaming::to_binary(source);

    // Collect back to Vec<Bytes>
    let items: Vec<_> = transformed.collect().await;
    let mut result = Vec::new();

    for item_result in items {
        match item_result {
            Ok(jeb_streaming::Item::Text(text)) => {
                result.push(Bytes::from(text.as_bytes().to_vec()));
            }
            Ok(jeb_streaming::Item::Bytes(bytes)) => {
                result.push(Bytes::from(bytes.to_vec()));
            }
            Ok(_) => {} // Skip other item types
            Err(e) => return Err(e.into()),
        }
    }
    Ok(result)
}
