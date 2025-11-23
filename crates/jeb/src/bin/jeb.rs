#![allow(unused)]
use {
    color_eyre::Error,
    derive_more::From,
    std::{
        collections::HashMap,
        io::{Read, Write},
        mem::{replace, take},
    },
    tap::Tap,
};

#[derive(Debug, Clone, Default)]
struct JsonRecord(indexmap::IndexMap<String, JsonValue>);

#[derive(Debug, Clone, Default)]
enum JsonValue {
    #[default]
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<JsonValue>),
    Object(JsonRecord),
}

#[derive(Debug, Clone)]
enum Item {
    Bytes(Vec<u8>),
    Record(JsonRecord),
}

pub fn main() {
    let mut args = Vec::<String>::from_iter(std::env::args());
    let own_path: String = args.remove(0);

    let commands = HashMap::<String, fn(&mut Vec<Item>) -> Result<(), Error>>::from_iter([
        ["stdin", |state: &mut Vec<Item>| -> Result<(), Error> {
            let mut buffer = Vec::<u8>::new();
            std::io::stdin().read_to_end(&mut buffer)?;
            state.push(Item::Bytes(buffer));
            Ok(())
        }],
        ["stdout", |state: &mut Vec<Item>| -> Result<(), Error> {
            for item in take(state) {
                match item {
                    Item::Bytes(blob) => {
                        std::io::stdout().write_all(&blob)?;
                    }
                    Item::Record(_) => {
                        return Err(Error::msg("cannot write non-bytes item to stdout"));
                    }
                }
            }
            Ok(())
        }],
    ]);

    if (args.is_empty() || args.iter().any(|arg| arg == "--help" || arg == "-h")) {
        eprint!("usage: {own_path} [--help|-h]");
        eprintln!();
        return;
    }

    let mut state = Vec::<Vec<u8>>::new();

    for arg in args {
        match arg.as_str() {
            "stdin" => {
                let mut buffer = Vec::<u8>::new();
                std::io::stdin().read_to_end(&mut buffer).unwrap();
                state.push(buffer);
            }

            "self" => {
                let own_data = std::fs::read(&own_path).unwrap();
                state.push(own_data)
            }

            "encode-z85" => {
                let input = take(&mut state).into_iter().flatten().collect::<Vec<u8>>();
                let encoded = jeb::encode_z85(&input);
                state.push(encoded);
            }

            _ => {
                eprintln!("error: unrecognized argument: {arg}");
                std::process::exit(1);
            }
        }
    }

    for blob in state {
        std::io::stdout().write_all(&blob).unwrap();
    }
}
