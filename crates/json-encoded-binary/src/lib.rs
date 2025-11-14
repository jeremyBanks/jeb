#![allow(unused)]

use nom_supreme::tag::streaming;
use nom_supreme::{error::ErrorTree, final_parser::final_parser, parser_ext::ParserExt};

mod byte_ranges;
mod const_checked;
mod errors;

pub use crate::byte_ranges::*;
pub use crate::const_checked::*;
pub use crate::errors::*;

pub const BASE_85: usize = 85;
pub const BLOCK_BYTES_4: usize = 4;
pub const BLOCK_DIGITS_5: usize = 5;

pub const RAW_PREFIX: u8 = b'|';
pub const RAW_PADDING: u8 = b'.';

pub const TARGET_LINE_SIZE_DIGITS: usize = 80;
pub const TARGET_LINE_SIZE_BYTES: usize = eq_usize(
    64,
    div_exact(TARGET_LINE_SIZE_DIGITS * BLOCK_BYTES_4, BLOCK_DIGITS_5),
);

/// We encode a maximum of 64 KiB of raw data per raw chunk.
pub const TARGET_RAW_BYTES: usize = eq_usize(65_536, 64 * 1024);
/// We encode a maximum of 16 Ki blocks per raw chunk.
pub const TARGET_RAW_BLOCKS: usize = eq_usize(16_384, div_exact(TARGET_RAW_BYTES, BLOCK_BYTES_4));

/// We decode the format's maximum of roughly 200 MiB of raw data per raw chunk.
pub const MAX_RAW_BYTES: usize = eq_usize(208_802_508, MAX_RAW_BLOCKS * BLOCK_BYTES_4);
/// The number of raw blocks in a raw chunk is limited by the maximum raw prefix
/// size value that can fit in the initial block with `RAW_PREFIX`.
pub const MAX_RAW_BLOCKS: usize = eq_usize(52_200_627, 2 + pow(BASE_85, BLOCK_DIGITS_5 - 1));

/// Encodes a 4-byte (32-bit) binary block into a 5-digit Z85 block.
pub const fn encode_z85_block(bytes: &[u8; BLOCK_BYTES_4]) -> [u8; BLOCK_DIGITS_5] {
    let mut encoded = [0u8; BLOCK_DIGITS_5];

    let mut value = u32::from_be_bytes(*bytes) as usize;

    let mut encoded_index = BLOCK_DIGITS_5 - 1;
    loop {
        let digit_value = value % BASE_85;
        value /= BASE_85;

        let digit = Z85[digit_value];
        encoded[encoded_index] = digit;

        if encoded_index > 0 {
            encoded_index -= 1;
            continue;
        } else {
            break;
        }
    }

    encoded
}

/// Decodes a 4-byte (32-bit) binary block into a 5-digit Z85 block.
///
/// Errors with `Panic` if an invalid digit is encountered or the value overflows.
pub const fn decode_z85_block(
    encoded: &[u8; BLOCK_DIGITS_5],
) -> Result<[u8; BLOCK_BYTES_4], &'static str> {
    let mut value: u32 = 0;

    let mut encoded_index = 0;
    loop {
        value = match value.checked_mul(BASE_85 as u32) {
            Some(value) => value,
            None => return Err("invalid overflowing value in decode_z85_block"),
        };

        let digit = encoded[encoded_index];
        let digit_value = Z85_LUT[digit as usize] as usize;

        if (digit_value >= BASE_85) {
            return Err("invalid Z85 digit in decode_z85_block");
        }
        value = match value.checked_add(digit_value as u32) {
            Some(value) => value,
            None => return Err("invalid overflowing value in decode_z85_block"),
        };

        if encoded_index < BLOCK_DIGITS_5 - 1 {
            encoded_index += 1;
            continue;
        } else {
            break;
        }
    }

    let bytes = (value as u32).to_be_bytes();

    Ok(bytes)
}

pub const fn decode_z85_block_or_panic(encoded: &[u8; BLOCK_DIGITS_5]) -> [u8; BLOCK_BYTES_4] {
    match decode_z85_block(encoded) {
        Ok(bytes) => bytes,
        Err(err) => panic!("{}", err),
    }
}

#[cfg(test)]
#[test]
fn test_z85_blocks() {
    macro_rules! assertions {
        () => {
            expect(b"00000", b"\x00\x00\x00\x00");
            expect(b"00001", b"\x00\x00\x00\x01");
            expect(b"0000#", b"\x00\x00\x00\x54");
            expect(b"00010", b"\x00\x00\x00\x55");
            expect(b"000##", b"\x00\x00\x1c\x38");
            expect(b"00100", b"\x00\x00\x1C\x39");
            expect(b"00###", b"\x00\x09\x5E\xEC");
            expect(b"01000", b"\x00\x09\x5E\xED");
            expect(b"0####", b"\x03\x1C\x84\xB0");
            expect(b"10000", b"\x03\x1C\x84\xB1");
            reject(b"#####");
            reject(b"#0000");
            reject(b"$0000");
            expect(b"%0000", b"\xFF\x22\x80\xB2");
            reject(b"%%%%%");
            expect(b"%nSc0", b"\xFF\xFF\xFF\xFF");
            reject(b"%nSc1");
            expect(b"%nSb#", b"\xFF\xFF\xFF\xFE");

            expect(b"01234", b"\x00\x09\x98\x62");
            expect(b"56789", b"\x0F\xC7\x99\x43");
            expect(b"abcde", b"\x1F\x85\x9A\x24");
            expect(b"fghij", b"\x2F\x43\x9B\x05");
            expect(b"klmno", b"\x3F\x01\x9B\xE6");
            expect(b"pqrst", b"\x4E\xBF\x9C\xC7");
            expect(b"uvwxy", b"\x5E\x7D\x9D\xA8");
            expect(b"zABCD", b"\x6E\x3B\x9E\x89");
            expect(b"EFGHI", b"\x7D\xF9\x9F\x6A");
            expect(b"JKLMN", b"\x8D\xB7\xA0\x4B");
            expect(b"OPQRS", b"\x9D\x75\xA1\x2C");
            expect(b"TUVWX", b"\xAD\x33\xA2\x0D");
            expect(b"YZ.-:", b"\xBC\xF1\xA2\xEE");
            expect(b"+=^!/", b"\xCC\xAF\xA3\xCF");
            expect(b"*?&<>", b"\xDC\x6D\xA4\xB0");
            expect(b"()[]{", b"\xEC\x2B\xA5\x91");
            expect(b"}@%$#", b"\xFB\xE9\xA6\x72");

            reject(b"     ");
            reject(b" 0000");
            reject(b"0 000");
            reject(b"00 00");
            reject(b"00|00");
            reject(b"00_00");
            reject(b"00,00");
            reject(b"00;00");
            reject(b"00~00");
            reject(b"00`00");
            reject(b"00'00");
            reject(b"00\"00");
            reject(b"00\\00");
            reject(b"000 0");
            reject(b"0000 ");
            reject(b"\0\0\0\0\0");
            reject(b"\n\n\n\n\n");
            reject(b"\xFF\xFF\xFF\xFF\xFF");
        };
    }

    {
        fn expect(encoded: &[u8; BLOCK_DIGITS_5], bytes: &[u8; BLOCK_BYTES_4]) {
            assert_eq!(Ok(bytes), decode_z85_block(encoded).as_ref());
            assert_eq!(encoded, &encode_z85_block(bytes));
        }

        fn reject(encoded: &[u8; BLOCK_DIGITS_5]) {
            assert!(decode_z85_block(encoded).is_err());
        }

        assertions!();
    }

    const _: () = {
        const fn expect(encoded: &[u8; BLOCK_DIGITS_5], bytes: &[u8; BLOCK_BYTES_4]) {
            eq_bytes(bytes, &decode_z85_block_or_panic(encoded));
            eq_bytes(encoded, &encode_z85_block(bytes));
        }

        const fn reject(encoded: &[u8; BLOCK_DIGITS_5]) {
            match decode_z85_block(encoded) {
                Ok(_) => panic!("expected error decoding invalid z85 block, but it succeeded"),
                Err(_) => {}
            }
        }

        assertions!();
    };
}
