use crate::{
    Panic,
    Z85,
    Z85_LUT,
};
/// This encoding uses base 85 for binary data.
pub const BASE_85: usize = 85;
/// This encoding works in 4-byte (32-bit) blocks.
pub const BLOCK_BYTES_4: usize = 4;
/// This encoding represents each block with 5 digits.
pub const BLOCK_DIGITS_5: usize = 5;
/// The number of blocks required to encode a given number of bytes.
pub const BLOCK_DIGITS_BY_BYTES: [usize; BLOCK_BYTES_4 + 1] = [0, 2, 3, 4, 5];
/// The number of bytes encoded by a given number of digits.
pub const BLOCK_BYTES_BY_DIGITS: [usize; BLOCK_DIGITS_5 + 1] = [0, -1 as _, 1, 2, 3, 4];
#[derive(Default)]
pub struct Encoder;
impl Encoder {
    #[must_use]
    pub fn encode_bytes(&self, _bytes: &[u8]) -> Vec<u8> {
        unimplemented!()
    }
}
#[derive(Default)]
pub struct Decoder;
impl Decoder {
    pub fn decode_bytes(&self, _encoded: &[u8]) -> Result<Vec<u8>, Panic> {
        unimplemented!()
    }
}
#[must_use]
pub fn encode(bytes: &[u8]) -> Vec<u8> {
    Encoder::default().encode_bytes(bytes)
}
pub fn decode(encoded: &[u8]) -> Result<Vec<u8>, Panic> {
    Decoder::default().decode_bytes(encoded)
}
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
            None => {
                return Err("decode_z85_block failed: invalid overflowing leading digit");
            }
        };
        let digit = encoded[encoded_index];
        let digit_value = Z85_LUT[digit as usize] as usize;
        if digit_value >= BASE_85 {
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
            use crate::bytes_eq;
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
#[must_use]
pub const fn encoded_z85_length(byte_length: usize) -> usize {
    let full_blocks = byte_length / BLOCK_BYTES_4;
    let remaining_bytes = byte_length % BLOCK_BYTES_4;
    let full_block_digits = full_blocks * BLOCK_DIGITS_5;
    let remaining_block_digits = BLOCK_DIGITS_BY_BYTES[remaining_bytes];
    full_block_digits + remaining_block_digits
}
#[must_use]
pub const fn decoded_z85_length(digit_length: usize) -> usize {
    let full_blocks = digit_length / BLOCK_DIGITS_5;
    let remaining_digits = digit_length % BLOCK_DIGITS_5;
    let full_block_bytes = full_blocks * BLOCK_BYTES_4;
    let remaining_block_bytes = BLOCK_BYTES_BY_DIGITS[remaining_digits];
    full_block_bytes + remaining_block_bytes
}
#[cfg_attr(
    feature = "wasm",
    wasm_bindgen::prelude::wasm_bindgen
)]
#[must_use]
pub fn encode_z85(bytes: &[u8]) -> Vec<u8> {
    let encoded_length = encoded_z85_length(bytes.len());
    let mut output = Vec::with_capacity(encoded_length);
    for bytes in bytes.chunks(BLOCK_BYTES_4) {
        let byte_length = bytes.len();
        let mut byte_block = [0x00; BLOCK_BYTES_4];
        byte_block[..byte_length].copy_from_slice(bytes);
        let encoded_length = BLOCK_DIGITS_BY_BYTES[bytes.len()];
        let encoded_block = encode_z85_block(byte_block);
        let encoded = &encoded_block[..encoded_length];
        output.extend_from_slice(encoded);
    }
    debug_assert!(output.len() == encoded_length);
    output
}
pub fn decode_z85(encoded: &[u8]) -> Result<Vec<u8>, crate::Panic> {
    let encoded: Vec<u8> = encoded
        .iter()
        .filter(|&&b| !b.is_ascii_whitespace())
        .copied()
        .collect();
    let decoded_length = decoded_z85_length(encoded.len());
    let mut output = Vec::with_capacity(decoded_length);
    for digits in encoded.chunks(BLOCK_DIGITS_5) {
        let digit_length = digits.len();
        let mut digit_block = [b'0'; BLOCK_DIGITS_5];
        digit_block[..digit_length].copy_from_slice(digits);
        let decoded_length = BLOCK_BYTES_BY_DIGITS[digits.len()];
        let decoded_block = decode_z85_block(digit_block)?;
        let decoded = &decoded_block[..decoded_length];
        output.extend_from_slice(decoded);
    }
    debug_assert!(output.len() == decoded_length);
    Ok(output)
}
