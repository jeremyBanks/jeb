pub mod z855;

pub use z855::{
    decode,
    decode_binary,
    encode,
    encode_with_options,
    z855_binary,
    DecodeError,
    EncodeError,
    Z855Options,
};

#[cfg(test)]
mod proptest;

#[cfg(test)]
mod proptest_priority1;

#[cfg(test)]
mod proptest_priority2;
