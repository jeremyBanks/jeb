#![allow(unused)]
use {
    bstr::BString,
    std::{
        io::{Read, Write},
        mem::{replace, take},
    },
    tap::Tap,
};

pub fn main() {
    let mut args = Vec::<String>::from_iter(std::env::args());
    let own_path: String = args.remove(0);


    if (args.is_empty() || args.iter().any(|arg| arg == "--help" || arg == "-h")) {
        eprintln!("usage: {own_path} [--help|-h]");
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
