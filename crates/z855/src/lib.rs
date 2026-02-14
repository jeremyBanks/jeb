pub mod z855;

pub use z855::{decode, encode, DecodeError};

#[cfg(test)]
mod proptest;

#[cfg(test)]
mod proptest_priority1;
