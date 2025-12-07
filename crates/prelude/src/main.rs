#![no_implicit_prelude]
use ::prelude::*;

#[allow(unused)]
pub fn main() {
    let x: integer = 42;
    let message = String::new();

    let test: bytes = b"hello, world!";
    let test = Bytes::from_iter(test.iter().copied());

    print!("Hello, world! {x}");
}
