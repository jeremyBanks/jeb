#![allow(unused)]

use nom_supreme::tag::streaming;
use nom_supreme::{error::ErrorTree, final_parser::final_parser, parser_ext::ParserExt};
use static_assertions::const_assert;

mod byte_ranges;
mod errors;

pub(crate) use crate::byte_ranges::*;
pub(crate) use crate::errors::*;

pub(crate) const BASE_85: usize = 85;
pub(crate) const BLOCK_BYTES_4: usize = 4;
pub(crate) const BLOCK_DIGITS_5: usize = 5;

pub(crate) const RAW_PREFIX: u8 = b'|';
pub(crate) const RAW_PADDING: u8 = b'.';

pub(crate) const TARGET_LINE_SIZE_DIGITS: usize = 80;
pub(crate) const TARGET_LINE_SIZE_BYTES: usize = eq(
    64,
    div(TARGET_LINE_SIZE_DIGITS * BLOCK_BYTES_4, BLOCK_DIGITS_5),
);

/// We encode a maximum of 64 KiB of raw data per raw chunk.
pub(crate) const TARGET_RAW_BYTES: usize = eq(65_536, 64 * 1024);
/// We encode a maximum of 16 Ki blocks per raw chunk.
pub(crate) const TARGET_RAW_BLOCKS: usize = eq(16_384, div(TARGET_RAW_BYTES, BLOCK_BYTES_4));

/// We decode the format's maximum of roughly 200 MiB of raw data per raw chunk.
pub(crate) const MAX_RAW_BYTES: usize = eq(208_802_508, MAX_RAW_BLOCKS * BLOCK_BYTES_4);
/// The number of raw blocks in a raw chunk is limited by the maximum raw prefix
/// size value that can fit in the initial block with `RAW_PREFIX`.
pub(crate) const MAX_RAW_BLOCKS: usize = eq(52_200_627, 2 + pow(BASE_85, BLOCK_DIGITS_5 - 1));

const fn eq(value: usize, calculation: usize) -> usize {
    if value != calculation {
        panic!("calculation did not match actual value");
    }
    value
}
const fn div(dividend: usize, divisor: usize) -> usize {
    if dividend % divisor != 0 {
        panic!("division left remainder");
    }
    dividend / divisor
}
const fn pow(base: usize, exponent: usize) -> usize {
    let mut result: usize = 1;
    let mut exp: usize = 0;
    while exp < exponent {
        result = result.checked_mul(base).expect("overflow in pow(...)");
        exp += 1;
    }
    result
}

pub const fn encode_z85_block(input: &[u8; BLOCK_BYTES_4]) -> [u8; BLOCK_DIGITS_5] {
    let mut value: u32 = 0;
    let mut index = 0;
    while index < BLOCK_BYTES_4 {
        let byte = input[index];
        value = (value << 8) | (byte as u32);
        index += 1;
    }
    let mut output = [0u8; BLOCK_DIGITS_5];
    let mut index = BLOCK_DIGITS_5;
    let base = BASE_85 as u32;
    while index > 0 {
        index -= 1;
        output[index] = (value % base) as u8 + 33;
        value /= base;
    }
    output
}

pub const fn decode_z85_block(input: &[u8; BLOCK_DIGITS_5]) -> [u8; BLOCK_BYTES_4] {
    let mut value: u32 = 0;
    let mut index = 0;
    let base = BASE_85 as u32;
    while index < BLOCK_DIGITS_5 {
        let digit = (input[index] as u32)
            .checked_sub(33)
            .expect("invalid z85 digit");
        if digit >= base {
            panic!("invalid z85 digit");
        }
        value = value
            .checked_mul(base)
            .expect("overflow in decode_z85_block(...)")
            .checked_add(digit)
            .expect("overflow in decode_z85_block(...)");
        index += 1;
    }
    let mut output = [0u8; BLOCK_BYTES_4];
    let mut index = BLOCK_BYTES_4;
    while index > 0 {
        index -= 1;
        output[index] = (value & 0xFF) as u8;
        value >>= 8;
    }
    output
}

const fn assert_eq(expected: &[u8], actual: &[u8]) -> bool {
    if expected.len() != actual.len() {
        panic!("asserted values were not the same (length mismatch)");
    }
    let mut index = 0;
    while index < expected.len() {
        if expected[index] != actual[index] {
            panic!("asserted values were not the same");
        }
    }
    true
}

const_assert! {
    assert_eq(b"00000", &encode_z85_block(b"\x00\x00\x00\x00"))
}

pub const S: [u8; 5] = encode_z85_block(b"\x00\x00\x00\x00");
