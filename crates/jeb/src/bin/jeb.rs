#![allow(unused)]
use {
    bytes::Bytes,
    color_eyre::Report,
    derive_more::From,
    std::{
        collections::HashMap,
        io::{Read, Write},
        mem::{replace, take},
    },
    tap::Tap,
};

#[derive(Debug, Clone, Default)]
struct JsonObject(indexmap::IndexMap<String, JsonValue>);

#[derive(Debug, Clone, Default)]
enum JsonValue {
    #[default]
    Null,
    Bool(bool),
    Unsigned(u64),
    Signed(i64),
    Float(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(JsonObject),
}

fn self_(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Report> {
    let own_path = std::env::current_exe()?;
    let own_data = std::fs::read(own_path)?;
    state.push(Bytes::from(own_data));
    Ok(state)
}

fn stdin(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Report> {
    let mut buffer = Vec::<u8>::new();
    std::io::stdin().read_to_end(&mut buffer)?;
    state.push(Bytes::from(buffer));
    Ok(state)
}

fn stdout(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Report> {
    for item in state {
        std::io::stdout().write_all(&item)?;
    }
    Ok(Vec::new())
}

fn encode_z85(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Report> {
    state.iter_mut().for_each(|b| *b = jeb::encode_z85(b).into());
    Ok(state)
}

fn first(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Report> {
    while (state.len() > 1) {
        state.pop();
    }
    Ok(state)
}

fn last(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Report> {
    let last = state.pop();;
    state.clear();
    if let Some(item) = last {
        state.push(item);
    }
    Ok(state)
}

fn split_lines(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Report> {
    let mut result = Vec::<Bytes>::new();
    for bytes in state {
        for line in bytes.split(|&byte| byte == b'\n') {
            result.push(Bytes::from(line.to_vec()));
        }
    }
    Ok(result)
}

fn split_64(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Report> {
    let mut result = Vec::<Bytes>::new();
    for bytes in state {
        let chunks = bytes.chunks(64);
        for chunk in chunks {
            result.push(Bytes::from(chunk.to_vec()));
        }
    }
    Ok(result)
}

fn split_80(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Report> {
    let mut result = Vec::<Bytes>::new();
    for bytes in state {
        let chunks = bytes.chunks(80);
        for chunk in chunks {
            result.push(Bytes::from(chunk.to_vec()));
        }
    }
    Ok(result)
}

fn split_64k(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Report> {
    let mut result = Vec::<Bytes>::new();
    for bytes in state {
        let chunks = bytes.chunks(65536);
        for chunk in chunks {
            result.push(Bytes::from(chunk.to_vec()));
        }
    }
    Ok(result)
}

fn join(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Report> {
    let input = take(&mut state).into_iter().flatten().collect::<Vec<u8>>();
    state.push(Bytes::from(input));
    Ok(state)
}

fn join_lines(mut state: Vec<Bytes>) -> Result<Vec<Bytes>, Report> {
    let input = take(&mut state)
        .into_iter()
        .flat_map(|b| b.into_iter().chain(std::iter::once(b'\n')))
        .collect::<Vec<u8>>();
    state.push(Bytes::from(input));
    Ok(state)
}

#[tokio::main(flavor = "current_thread")]
pub async fn main() -> Result<(), Report> {
    let mut args = Vec::<String>::from_iter(std::env::args());
    let own_path: String = args.remove(0);

    if (args.is_empty() || args.iter().any(|arg| arg == "--help" || arg == "-h")) {
        eprint!("usage: {own_path} [--help|-h]");
        eprintln!();
        return Ok(());
    }

    let mut state = Vec::<Bytes>::new();

    eprintln!("{own_path} {}", args.join(" "));

    for arg in args {
        state = match arg.as_str() {
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
            "encode-z85" => encode_z85(state)?,
            arg => {
                eprintln!("error: unrecognized argument: {arg}");
                std::process::exit(1);
            }
        }
    }

    println!();

    Ok(())
}
