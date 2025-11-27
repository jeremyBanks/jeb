use {
    jeb::{Panic, model::Bytes},
    owo_colors::OwoColorize,
    regex::Regex,
    std::{
        convert::Infallible,
        io::{Read, Write},
        mem::take,
        sync::LazyLock,
    },
};


#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), Infallible> {
    inner_main().await.ok();
    Ok(())
}

pub async fn inner_main() -> Result<(), Panic> {
    let mut args = Vec::<String>::from_iter(std::env::args());
    let own_path: String = args.remove(0);

    args = args
        .into_iter()
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
            "split-lines" => split_lines(state)?,
            "join" => join(state)?,
            "join-lines" => join_lines(state)?,
            "join-space" => join_space(state)?,
            "collapse" => collapse(state)?,
            "filter" => filter(state)?,
            "encode-z85" => encode_z85(state)?,
            "encode-jeb85" => encode_jeb85(state)?,
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
            _ => {
                if command.starts_with(".") || command.starts_with("/") {
                    read(state, &command)?
                } else if let Some(arg) = command.strip_prefix("last-") {
                    last_n(state, arg)?
                } else if let Some(arg) = command.strip_prefix("first-") {
                    first_n(state, arg)?
                } else if let Some(arg) = command.strip_prefix("split-") {
                    split_n(state, arg)?
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
        let encoded = jeb::encode_z85(&bytes);
        *piece = encoded.into();
    }
    Ok(state)
}

fn encode_jeb85(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    for piece in &mut state {
        let bytes = take(piece);
        let encoded = jeb::encode_jeb85(&bytes);
        *piece = encoded.into();
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

fn collapse(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    let input = state.pop().unwrap();
    let mut output = Vec::<u8>::new();
    let mut in_whitespace = false;
    for &byte in &input {
        if byte.is_ascii_whitespace() {
            in_whitespace = true;
        } else {
            if in_whitespace {
                output.push(b' ');
                in_whitespace = false;
            }
            output.push(byte);
        }
    }
    state.push(Bytes::from(output));
    Ok(state)
}

fn split_lines(state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    let mut result = Vec::<Bytes>::new();
    for bytes in state {
        for line in bytes.split(|&byte| byte == b'\n') {
            result.push(Bytes::from(line.to_vec()));
        }
    }
    Ok(result)
}

fn split_n(state: Vec<Bytes>, arg: &str) -> Result<Vec<Bytes>, Panic> {
    let rest = arg.to_ascii_uppercase();
    let mut rest = rest.as_str();

    let mut unit = 1;

    let binary;
    if let Some(_next) = rest.strip_suffix("IB") {
        binary = true;
    } else if let Some(_next) = rest.strip_suffix("B") {
        binary = false;
    } else if let Some(_next) = rest.strip_suffix("I") {
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

    let mut result = Vec::<Bytes>::new();
    for bytes in state {
        let chunks = bytes.chunks(size);
        for chunk in chunks {
            result.push(Bytes::from(chunk.to_vec()));
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

fn filter(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    Ok(take(&mut state)
        .into_iter()
        .filter(|bytes| !bytes.is_empty())
        .collect())
}
