#![warn(
    clippy::std_instead_of_core,
    clippy::pedantic,
    clippy::cargo,
    clippy::nursery,
    clippy::allow_attributes,
    clippy::arbitrary_source_item_ordering
)]
#![expect(
    unused,
    missing_docs,
    clippy::missing_errors_doc,
    clippy::redundant_else,
    clippy::needless_continue,
    clippy::manual_assert,
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::too_long_first_doc_paragraph,
    clippy::arbitrary_source_item_ordering
)]
// cSpell:ignoreRegExp b"(\\?.){5}"

mod byte_ranges;
mod const_checked;
mod errors;
pub mod jeb85;

use {
    core::fmt::Debug,
    nom_supreme::{
        error::ErrorTree, final_parser::final_parser, parser_ext::ParserExt, tag::streaming,
    },
};

pub use crate::{
    byte_ranges::*,
    const_checked::*,
    errors::*,
    jeb85::{decode_jeb85, encode_jeb85},
};

// Aliases for compatibility with slop crate
pub const Z85_ALPHABET: &[u8; 85] = Z85;
pub const Z85_DECODE: [u8; 256] = Z85_LUT;
pub const MAX_TEXT_SIZE: usize = TARGET_RAW_BYTES;

// MARK: encoding constants

/// This encoding uses base 85 for binary data.
pub const BASE_85: usize = 85;
/// This encoding works in 4-byte (32-bit) blocks.
pub const BLOCK_BYTES_4: usize = 4;
/// This encoding represents each block with 5 digits.
pub const BLOCK_DIGITS_5: usize = 5;

/// The prefix byte preceding raw data.
pub const RAW_PREFIX: u8 = b'_';
/// The padding byte following raw data as necessary to align following blocks.
pub const RAW_PADDING: u8 = b'_';

/// This encoding allows maximum of roughly 200 MiB of raw data per raw chunk.
pub const MAX_RAW_BYTES: usize = usize_eq(208_802_508, MAX_RAW_BLOCKS * BLOCK_BYTES_4);
/// This encoding's number of raw blocks in a raw chunk is limited by the
/// maximum raw prefix size value that can fit in the initial block with
/// `RAW_PREFIX`.
pub const MAX_RAW_BLOCKS: usize = usize_eq(52_200_627, 2 + pow(BASE_85, BLOCK_DIGITS_5 - 1));

/// The number of blocks required to encode a given number of bytes.
pub const BLOCK_DIGITS_BY_BYTES: [usize; BLOCK_BYTES_4 + 1] = [0, 2, 3, 4, 5];
/// The number of bytes encoded by a given number of digits.
pub const BLOCK_BYTES_BY_DIGITS: [usize; BLOCK_DIGITS_5 + 1] = [0, -1 as _, 1, 2, 3, 4];

/// When this encoding is used to convert binary data into line of text, our
/// implementation limits each line to 80 digits.
pub const TARGET_LINE_SIZE_DIGITS: usize = 80;
/// When this encoding is split into 80 digit lines, each line contains 64 bytes
/// of data, which has a good chance of some alignment with binary data.
pub const TARGET_LINE_SIZE_BYTES: usize = usize_eq(
    64,
    div_exact(TARGET_LINE_SIZE_DIGITS * BLOCK_BYTES_4, BLOCK_DIGITS_5),
);

/// We encode a maximum of 64 KiB of raw data per raw chunk.
pub const TARGET_RAW_BYTES: usize = usize_eq(65_536, 64 * 1024);
/// We encode a maximum of 16 Ki blocks per raw chunk.
pub const TARGET_RAW_BLOCKS: usize = usize_eq(16_384, div_exact(TARGET_RAW_BYTES, BLOCK_BYTES_4));

// MARK: high-level interface

#[derive(Default)]
pub struct Encoder;
impl Encoder {
    /// Encodes bytes using Z85 encoding.
    ///
    /// Encodes 4-byte blocks into 5 Z85 characters. Partial blocks at the end
    /// are right-aligned and use fewer characters.
    #[must_use]
    pub fn encode_bytes(bytes: &[u8]) -> Vec<u8> {
        let mut result =
            Vec::with_capacity((bytes.len() * BLOCK_DIGITS_5) / BLOCK_BYTES_4 + BLOCK_DIGITS_5);

        let mut i = 0;
        // Encode full 4-byte blocks
        while i + BLOCK_BYTES_4 <= bytes.len() {
            let mut block = [0u8; BLOCK_BYTES_4];
            block.copy_from_slice(&bytes[i..i + BLOCK_BYTES_4]);
            let encoded = encode_z85_block(block);
            result.extend_from_slice(&encoded);
            i += BLOCK_BYTES_4;
        }

        // Handle partial block at end (1-3 bytes remaining)
        let remaining = bytes.len() - i;
        if remaining > 0 {
            // Pad on the right (little-endian for partial blocks)
            let mut block = [0u8; BLOCK_BYTES_4];
            // Put bytes at the END of the block (right-aligned)
            let start_pos = BLOCK_BYTES_4 - remaining;
            block[start_pos..].copy_from_slice(&bytes[i..]);
            let encoded = encode_z85_block(block);
            // Use the LAST N digits (right-aligned)
            let digits_needed = BLOCK_DIGITS_BY_BYTES[remaining];
            let start_digit = BLOCK_DIGITS_5 - digits_needed;
            result.extend_from_slice(&encoded[start_digit..]);
        }

        result
    }
}

#[derive(Default)]
pub struct Decoder;

impl Decoder {
    /// Decodes Z85-encoded bytes back to original binary data.
    ///
    /// # Panics
    ///
    /// Panics if the encoded data contains invalid Z85 characters or an invalid
    /// number of digits in a partial block.
    #[must_use]
    pub fn decode_bytes(encoded: &[u8]) -> Vec<u8> {
        let mut result = Vec::with_capacity((encoded.len() * BLOCK_BYTES_4) / BLOCK_DIGITS_5);

        let mut i = 0;
        // Decode full 5-digit blocks
        while i + BLOCK_DIGITS_5 <= encoded.len() {
            let mut block = [0u8; BLOCK_DIGITS_5];
            block.copy_from_slice(&encoded[i..i + BLOCK_DIGITS_5]);
            let decoded = match decode_z85_block(block) {
                Ok(bytes) => bytes,
                Err(e) => panic!("{}", e),
            };
            result.extend_from_slice(&decoded);
            i += BLOCK_DIGITS_5;
        }

        // Handle partial block at end (2-4 digits remaining)
        let remaining = encoded.len() - i;
        if remaining > 0 {
            // Pad with '0' at the START (left-aligned)
            let mut block = [b'0'; BLOCK_DIGITS_5];
            let start_pos = BLOCK_DIGITS_5 - remaining;
            block[start_pos..].copy_from_slice(&encoded[i..]);
            let decoded = match decode_z85_block(block) {
                Ok(bytes) => bytes,
                Err(e) => panic!("{}", e),
            };
            // Use the LAST N bytes (right-aligned)
            let bytes_decoded = BLOCK_BYTES_BY_DIGITS[remaining];
            if bytes_decoded == usize::MAX {
                panic!("Invalid number of digits in partial block");
            }
            let start_byte = BLOCK_BYTES_4 - bytes_decoded;
            result.extend_from_slice(&decoded[start_byte..]);
        }

        result
    }
}

#[must_use]
pub fn encode(bytes: &[u8]) -> Vec<u8> {
    Encoder::encode_bytes(bytes)
}

#[must_use]
pub fn decode(encoded: &[u8]) -> Vec<u8> {
    Decoder::decode_bytes(encoded)
}

// MARK: Z85 block ser/de

/// Encodes a 4-byte (32-bit) binary block into a 5-digit Z85 block.
#[must_use]
pub const fn encode_z85_block(bytes: [u8; BLOCK_BYTES_4]) -> [u8; BLOCK_DIGITS_5] {
    let mut encoded = [0u8; BLOCK_DIGITS_5];

    let mut value = u32::from_be_bytes(bytes) as usize;

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

/// Decodes a 5-digit Z85 block into a 4-byte (32-bit) binary block.
///
/// Errors with `Panic` if an invalid digit is encountered or the value
/// overflows.
pub const fn decode_z85_block(
    encoded: [u8; BLOCK_DIGITS_5],
) -> Result<[u8; BLOCK_BYTES_4], &'static str> {
    let mut value: u32 = 0;

    let mut encoded_index = 0;
    loop {
        value = match value.checked_mul(BASE_85 as u32) {
            Some(value) => value,
            None => return Err("decode_z85_block failed: invalid overflowing leading digit"),
        };

        let digit = encoded[encoded_index];
        let digit_value = Z85_LUT[digit as usize] as usize;

        if (digit_value >= BASE_85) {
            return Err("decode_z85_block failed: invalid digit");
        }

        value = match value.checked_add(digit_value as u32) {
            Some(value) => value,
            None => return Err("decode_z85_block failed: invalid overflowing value"),
        };

        if encoded_index < BLOCK_DIGITS_5 - 1 {
            encoded_index += 1;
            continue;
        } else {
            break;
        }
    }

    let bytes = value.to_be_bytes();

    Ok(bytes)
}

/// Decodes a Z85 block, panicking on error.
///
/// # Panics
///
/// Panics if the encoded block contains invalid digits or the value overflows.
#[must_use]
pub const fn decode_z85_block_or_panic(encoded: [u8; BLOCK_DIGITS_5]) -> [u8; BLOCK_BYTES_4] {
    match decode_z85_block(encoded) {
        Ok(bytes) => bytes,
        Err(err) => panic!("{}", err),
    }
}

#[cfg(test)]
#[test]
#[expect(clippy::trivially_copy_pass_by_ref)]
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

    const _: () = {
        const fn expect(encoded: &[u8; BLOCK_DIGITS_5], bytes: &[u8; BLOCK_BYTES_4]) {
            bytes_eq(bytes, &decode_z85_block_or_panic(*encoded));
            bytes_eq(encoded, &encode_z85_block(*bytes));
        }

        const fn reject(encoded: &[u8; BLOCK_DIGITS_5]) {
            if decode_z85_block(*encoded).is_ok() {
                panic!("expected error decoding invalid z85 block, but it succeeded")
            }
        }

        assertions!();
    };

    {
        fn expect(encoded: &[u8; BLOCK_DIGITS_5], bytes: &[u8; BLOCK_BYTES_4]) {
            assert_eq!(Ok(bytes), decode_z85_block(*encoded).as_ref());
            assert_eq!(encoded, &encode_z85_block(*bytes));
        }

        fn reject(encoded: &[u8; BLOCK_DIGITS_5]) {
            assert!(decode_z85_block(*encoded).is_err());
        }

        assertions!();
    }
}


use std::{
    io::{self, Read, Write},
    process,
};

/// A binary stream with an associated error flag
#[derive(Clone, Debug)]
struct BinaryStream {
    data: Vec<u8>,
    error: bool,
}

impl BinaryStream {
    fn new(data: Vec<u8>) -> Self {
        Self { data, error: false }
    }
}

/// A stream of binary streams
#[derive(Debug)]
struct StreamOfStreams {
    streams: Vec<BinaryStream>,
}

impl StreamOfStreams {
    fn from_single(data: Vec<u8>) -> Self {
        Self {
            streams: vec![BinaryStream::new(data)],
        }
    }

    fn has_errors(&self) -> bool {
        self.streams.iter().any(|s| s.error)
    }

    fn to_single(self) -> io::Result<BinaryStream> {
        if self.streams.len() != 1 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Pipeline ended with stream-of-streams (forgot to join?)",
            ));
        }
        Ok(self.streams.into_iter().next().unwrap())
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // Read all input from stdin
    let mut input = Vec::new();
    if let Err(e) = io::stdin().read_to_end(&mut input) {
        eprintln!("Error reading stdin: {}", e);
        process::exit(1);
    }

    // Process through pipeline
    match process_pipeline(&args, input) {
        Ok(output_stream) => {
            // Write to stdout
            if let Err(e) = io::stdout().write_all(&output_stream.data) {
                eprintln!("Error writing to stdout: {}", e);
                process::exit(1);
            }

            // Exit with error code if stream has error flag
            if output_stream.error {
                process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    }
}

fn process_pipeline(commands: &[String], input: Vec<u8>) -> io::Result<BinaryStream> {
    let mut streams = StreamOfStreams::from_single(input);

    for (cmd_idx, cmd) in commands.iter().enumerate() {
        let mut error_logged_this_command = false;

        match cmd.as_str() {
            "encode-z85" => {
                streams.streams = streams
                    .streams
                    .into_iter()
                    .map(|stream| BinaryStream {
                        data: crate::encode(&stream.data),
                        error: stream.error, // Propagate error flag
                    })
                    .collect();
            }
            "decode-z85" => {
                streams.streams = streams
                    .streams
                    .into_iter()
                    .enumerate()
                    .map(|(stream_idx, stream)| {
                        let (decoded, had_error) = decode_z85_with_errors(&stream.data);
                        if had_error && !error_logged_this_command {
                            eprintln!(
                                "[cmd {} (decode-z85)]: error in stream {}",
                                cmd_idx, stream_idx
                            );
                            error_logged_this_command = true;
                        }
                        BinaryStream {
                            data: decoded,
                            error: stream.error || had_error, // Propagate or set error
                        }
                    })
                    .collect();
            }
            "encode-jeb85" => {
                streams.streams = streams
                    .streams
                    .into_iter()
                    .map(|stream| BinaryStream {
                        data: crate::encode_jeb85(&stream.data),
                        error: stream.error,
                    })
                    .collect();
            }
            "decode-jeb85" => {
                streams.streams = streams
                    .streams
                    .into_iter()
                    .enumerate()
                    .map(|(stream_idx, stream)| {
                        let (decoded, had_error) = decode_jeb85_with_errors(&stream.data);
                        if had_error && !error_logged_this_command {
                            eprintln!(
                                "[cmd {} (decode-jeb85)]: error in stream {}",
                                cmd_idx, stream_idx
                            );
                            error_logged_this_command = true;
                        }
                        BinaryStream {
                            data: decoded,
                            error: stream.error || had_error,
                        }
                    })
                    .collect();
            }
            "split-64K" => {
                // Split each stream into 64 KiB chunks
                const CHUNK_SIZE: usize = 64 * 1024;
                let mut new_streams = Vec::new();
                for stream in streams.streams {
                    for chunk in stream.data.chunks(CHUNK_SIZE) {
                        new_streams.push(BinaryStream {
                            data: chunk.to_vec(),
                            error: stream.error, // Propagate error to all chunks
                        });
                    }
                }
                streams.streams = new_streams;
            }
            "split-80" => {
                // Split each stream into 80 byte chunks
                const CHUNK_SIZE: usize = 80;
                let mut new_streams = Vec::new();
                for stream in streams.streams {
                    for chunk in stream.data.chunks(CHUNK_SIZE) {
                        new_streams.push(BinaryStream {
                            data: chunk.to_vec(),
                            error: stream.error, // Propagate error to all chunks
                        });
                    }
                }
                streams.streams = new_streams;
            }
            "split-64" => {
                // Split each stream into 64 byte chunks
                const CHUNK_SIZE: usize = 64;
                let mut new_streams = Vec::new();
                for stream in streams.streams {
                    for chunk in stream.data.chunks(CHUNK_SIZE) {
                        new_streams.push(BinaryStream {
                            data: chunk.to_vec(),
                            error: stream.error, // Propagate error to all chunks
                        });
                    }
                }
                streams.streams = new_streams;
            }
            "first" => {
                // Keep only the first substream
                if !streams.streams.is_empty() {
                    let first = streams.streams.into_iter().next().unwrap();
                    streams.streams = vec![first];
                }
            }
            "split-lines" => {
                // Split each stream by newlines
                let mut new_streams = Vec::new();
                for stream in streams.streams {
                    for line in stream.data.split(|&b| b == b'\n') {
                        if !line.is_empty() {
                            new_streams.push(BinaryStream {
                                data: line.to_vec(),
                                error: stream.error, // Propagate error
                            });
                        }
                    }
                }
                streams.streams = new_streams;
            }
            "join" => {
                // Join all streams with no delimiter
                let has_error = streams.has_errors();
                let data: Vec<u8> = streams.streams.into_iter().flat_map(|s| s.data).collect();
                streams = StreamOfStreams {
                    streams: vec![BinaryStream {
                        data,
                        error: has_error,
                    }],
                };
            }
            "join-lines" => {
                // Join all streams with newlines
                let has_error = streams.has_errors();
                let mut data = Vec::new();
                let stream_count = streams.streams.len();
                for (i, stream) in streams.streams.into_iter().enumerate() {
                    data.extend_from_slice(&stream.data);
                    if i < stream_count - 1 {
                        data.push(b'\n');
                    }
                }
                streams = StreamOfStreams {
                    streams: vec![BinaryStream {
                        data,
                        error: has_error,
                    }],
                };
            }
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Unknown command: {}", cmd),
                ));
            }
        }
    }

    streams.to_single()
}

/// Decode Z85 with error handling
/// Returns (output, had_error)
fn decode_z85_with_errors(input: &[u8]) -> (Vec<u8>, bool) {
    let mut output = Vec::new();
    let mut had_error = false;
    let mut i = 0;

    while i + 5 <= input.len() {
        let block = [
            input[i],
            input[i + 1],
            input[i + 2],
            input[i + 3],
            input[i + 4],
        ];

        match crate::decode_z85_block(block) {
            Ok(decoded) => {
                output.extend_from_slice(&decoded);
            }
            Err(_) => {
                // Error: output "E(" + original 5 bytes + ")" = 8 bytes
                output.extend_from_slice(b"E(");
                output.extend_from_slice(&block);
                output.push(b')');
                had_error = true;
            }
        }
        i += 5;
    }

    // Handle remaining bytes (partial block)
    if i < input.len() {
        let remaining = input.len() - i;
        let mut block = [b'0'; 5];
        let start_pos = 5 - remaining;
        block[start_pos..].copy_from_slice(&input[i..]);

        match crate::decode_z85_block(block) {
            Ok(decoded) => {
                let bytes_decoded = crate::BLOCK_BYTES_BY_DIGITS[remaining];
                if bytes_decoded != usize::MAX {
                    let start_byte = 4 - bytes_decoded;
                    output.extend_from_slice(&decoded[start_byte..]);
                }
            }
            Err(_) => {
                // Error on partial block: output "E(" + original bytes + ")" (padded to 8
                // total)
                output.extend_from_slice(b"E(");
                output.extend_from_slice(&input[i..]);
                // Pad to make total 8 bytes (2 + remaining + padding + 1)
                let padding_needed = 5 - remaining;
                output.extend(std::iter::repeat_n(b'0', padding_needed));
                output.push(b')');
                had_error = true;
            }
        }
    }

    (output, had_error)
}

/// Decode JEB85 with error handling
/// Returns (output, had_error)
fn decode_jeb85_with_errors(input: &[u8]) -> (Vec<u8>, bool) {
    // For now, JEB85 decode uses same error handling as Z85
    // If input contains . markers, it's binary mode
    // If it's text mode, it can't really fail

    if input.is_empty() {
        return (Vec::new(), false);
    }

    // Check if it's text mode (no . markers and text-safe)
    if !input.contains(&b'_') {
        let all_z85 = input
            .iter()
            .all(|&b| crate::Z85_DECODE[b as usize] != 255);

        // Simple check for text-safe
        if let Ok(s) = core::str::from_utf8(input)
            && !all_z85 && s.len() <= 65536 {
                // Text mode passthrough
                return (input.to_vec(), false);
            }
    }

    // Binary mode - for now just use the non-error version
    // TODO: Implement proper error handling for JEB85 binary mode
    (crate::decode_jeb85(input), false)
}
