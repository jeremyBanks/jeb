#![allow(unused)]

use {
    jeb::{Panic, model::Bytes},
    owo_colors::{OwoColorize, colors::*},
    std::{
        collections::HashMap,
        fmt::Debug,
        io::{Read, Write},
        mem::{replace, take},
    },
    tap::Tap,
};


static README: &str = include_str!("../../../../README.md");

#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), ()> {
    inner_main().await.map_err(|_| ())
}

pub async fn inner_main() -> Result<(), Panic> {
    let mut args = Vec::<String>::from_iter(std::env::args());
    let own_path: String = args.remove(0);

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

    for command in commands {
        state = match command.as_str() {
            "help" | "--help" | "-h" | "-?" => help(state)?,
            "stdin" => stdin(state)?,
            "stdout" => stdout(state)?,
            "self" => self_(state)?,
            "first" => first(state)?,
            "last" => last(state)?,
            "split-lines" => split_lines(state)?,
            "split-64" => split_64(state)?,
            "split-80" => split_80(state)?,
            "split-64k" => split_64k(state)?,
            "join" => join(state)?,
            "join-lines" => join_lines(state)?,
            "join-space" => join_space(state)?,
            "filter" => filter(state)?,
            "encode-z85" => encode_z85(state)?,
            "encode-jeb85" => encode_jeb85(state)?,
            arg => {
                if command.starts_with(".") || command.starts_with("/") {
                    read(state, &command)?
                // } else if (command.starts_with("http://") || command.starts_with("https://")) {
                // fetch(state, &command).await?
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
    state.push(README.into());

    Ok(state)
}

fn read(mut state: Vec<Bytes>, path: &str) -> Result<Vec<Bytes>, Panic> {
    let data = std::fs::read(path)?;
    state.push(Bytes::from(data));
    Ok(state)
}

// async fn fetch(mut state: Vec<Bytes>, url: &str) -> Result<Vec<Bytes>,
// Panic> {     let response = reqwest::get(url).await?;
//     response.error_for_status_ref()?;
//     let bytes = response.bytes().await?;
//     state.push(Bytes::from(bytes.to_vec()));
//     Ok(state)
// }

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

fn stdout(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    for item in state {
        std::io::stdout().write_all(&item)?;
    }
    Ok(Vec::new())
}

fn encode_z85(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    for piece in &mut state {
        let bytes = take(piece);
        let encoded = jeb::encode_z85(&bytes);
        *piece = encoded;
    }
    Ok(state)
}

fn encode_jeb85(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    for piece in &mut state {
        let bytes = take(piece);
        let encoded = jeb::encode_jeb85(&bytes);
        *piece = encoded;
    }
    Ok(state)
}

fn first(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    while (state.len() > 1) {
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

fn split_lines(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    let mut result = Vec::<Bytes>::new();
    for bytes in state {
        for line in bytes.split(|&byte| byte == b'\n') {
            result.push(Bytes::from(line.to_vec()));
        }
    }
    Ok(result)
}

fn split_64(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    let mut result = Vec::<Bytes>::new();
    for bytes in state {
        let chunks = bytes.chunks(64);
        for chunk in chunks {
            result.push(Bytes::from(chunk.to_vec()));
        }
    }
    Ok(result)
}

fn split_80(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    let mut result = Vec::<Bytes>::new();
    for bytes in state {
        let chunks = bytes.chunks(80);
        for chunk in chunks {
            result.push(Bytes::from(chunk.to_vec()));
        }
    }
    Ok(result)
}

fn split_64k(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Panic> {
    let mut result = Vec::<Bytes>::new();
    for bytes in state {
        let chunks = bytes.chunks(65536);
        for chunk in chunks {
            result.push(Bytes::from(chunk.to_vec()));
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
        .collect::<Vec<Vec<u8>>>())
}
