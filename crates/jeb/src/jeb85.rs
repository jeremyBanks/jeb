use crate::{
    ASCII_INLINE_TEXT_LUT,
    Z85,
    div_exact,
    pow,
    usize_eq,
    z85::{
        BASE_85,
        BLOCK_BYTES_4,
        BLOCK_DIGITS_5,
        BLOCK_DIGITS_BY_BYTES,
        encode_z85_block,
        encoded_z85_length,
    },
};
/// The prefix byte preceding raw data.
pub const RAW_PREFIX: u8 = b'|';
/// The default padding byte repeated after raw data to align following blocks.
pub const RAW_PADDING: u8 = b'.';
/// This encoding allows maximum of roughly 200 MiB of raw data per raw chunk.
pub const MAX_RAW_BYTES: usize = usize_eq(208_802_508, MAX_RAW_BLOCKS * BLOCK_BYTES_4);
/// This encoding's number of raw blocks in a raw chunk is limited by the
/// maximum raw prefix size value that can fit in the initial block with
/// `RAW_PREFIX`.
pub const MAX_RAW_BLOCKS: usize = usize_eq(52_200_627, 2 + pow(BASE_85, BLOCK_DIGITS_5 - 1));
/// We encode a maximum of 64 KiB of raw data per raw chunk.
pub const TARGET_RAW_BYTES: usize = usize_eq(65_536, 64 * 1024);
/// We encode a maximum of 16 Ki blocks per raw chunk.
pub const TARGET_RAW_BLOCKS: usize = usize_eq(16_384, div_exact(TARGET_RAW_BYTES, BLOCK_BYTES_4));
/// When this encoding is used to convert binary data into line of text, our
/// implementation limits each line to 80 digits.
pub const TARGET_LINE_SIZE_DIGITS: usize = 80;
/// When this encoding is split into 80 digit lines, each line contains 64 bytes
/// of data, which has a good chance of some alignment with binary data.
pub const TARGET_LINE_SIZE_BYTES: usize = usize_eq(
    64,
    div_exact(TARGET_LINE_SIZE_DIGITS * BLOCK_BYTES_4, BLOCK_DIGITS_5),
);
#[cfg_attr(
    feature = "wasm",
    wasm_bindgen::prelude::wasm_bindgen
)]
#[must_use]
pub fn encode_jeb85(bytes: &[u8]) -> Vec<u8> {
    let encoded_length = encoded_z85_length(bytes.len());
    let mut output = Vec::with_capacity(encoded_length);
    let mut raw_buffer = Vec::<u8>::new();
    for bytes in bytes.chunks(BLOCK_BYTES_4) {
        if bytes.iter().all(|b| ASCII_INLINE_TEXT_LUT[*b as usize]) {
            raw_buffer.extend_from_slice(bytes);
            continue;
        }
        if !raw_buffer.is_empty() {
            let raw_block_count = raw_buffer.len() / BLOCK_BYTES_4;
            if raw_block_count == 1 {
                output.extend([RAW_PREFIX]);
                output.extend(&raw_buffer);
            } else {
                let block_count_prefix_value = raw_block_count - 1;
                let block_count_prefix_block = encode_z85_block(
                    u32::try_from(block_count_prefix_value)
                        .unwrap()
                        .to_be_bytes(),
                );
                let mut block_count_prefix = &block_count_prefix_block[..];
                while block_count_prefix.first() == Some(&b'0') {
                    block_count_prefix = &block_count_prefix[1..];
                }
                let mut block_prefix = block_count_prefix.to_vec();
                block_prefix.extend([RAW_PREFIX]);
                let raw_block_digits = raw_block_count * BLOCK_DIGITS_5;
                let padding_needed = raw_block_digits - block_prefix.len() - raw_buffer.len();
                let mut padding = vec![RAW_PADDING; padding_needed];
                let mut cosmetic_padding = Vec::new();
                for byte in bytes {
                    if ASCII_INLINE_TEXT_LUT[*byte as usize] {
                        cosmetic_padding.push(*byte);
                    } else {
                        break;
                    }
                }
                cosmetic_padding.push(RAW_PREFIX);
                let available_len = padding.len().min(cosmetic_padding.len());
                padding[..available_len].copy_from_slice(&cosmetic_padding[..available_len]);
                output.extend(&block_prefix);
                output.extend(&raw_buffer);
                output.extend(&padding);
            }
            raw_buffer.clear();
        }
        let byte_length = bytes.len();
        let mut byte_block = [0x00; BLOCK_BYTES_4];
        byte_block[..byte_length].copy_from_slice(bytes);
        let encoded_length = BLOCK_DIGITS_BY_BYTES[bytes.len()];
        let encoded_block = encode_z85_block(byte_block);
        let encoded = &encoded_block[..encoded_length];
        output.extend_from_slice(encoded);
    }
    if !raw_buffer.is_empty() {
        if raw_buffer.len() > BLOCK_BYTES_4 {
            output.push(Z85[0]);
        }
        output.push(RAW_PREFIX);
        output.extend_from_slice(&raw_buffer);
        raw_buffer.clear();
    }
    debug_assert!(output.len() <= encoded_length);
    output
}
