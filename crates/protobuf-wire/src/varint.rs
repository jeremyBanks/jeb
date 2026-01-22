//! Varint encoding and decoding for the protobuf wire format.
//!
//! Varints encode unsigned 64-bit integers using 1-10 bytes. Each byte uses
//! 7 bits for payload and 1 bit (MSB) as a continuation flag.

use crate::ParseError;

/// Maximum number of bytes in a valid varint (for u64).
const MAX_VARINT_BYTES: usize = 10;

/// Decode a varint from a byte slice.
///
/// Returns the decoded value and the number of bytes consumed.
/// Non-canonical encodings (extra zero bytes) are accepted but logged as warnings.
pub fn decode_varint(bytes: &[u8]) -> Result<(u64, usize), ParseError> {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;

    for (i, &byte) in bytes.iter().enumerate() {
        if i >= MAX_VARINT_BYTES {
            return Err(ParseError::InvalidVarint);
        }

        // Extract the 7-bit payload
        let payload = (byte & 0x7F) as u64;

        // Check for overflow on the last byte
        if i == MAX_VARINT_BYTES - 1 && byte > 0x01 {
            return Err(ParseError::InvalidVarint);
        }

        result |= payload << shift;
        shift += 7;

        // Check continuation bit
        if byte & 0x80 == 0 {
            // Check for non-canonical encoding (trailing zeros)
            // A varint is non-canonical if it has continuation bytes with zero payload
            // that don't contribute to the value
            let canonical_len = canonical_varint_len(result);
            if i + 1 > canonical_len {
                log::warn!(
                    "Non-canonical varint encoding: {} bytes used for value {} (canonical: {} bytes)",
                    i + 1,
                    result,
                    canonical_len
                );
            }

            return Ok((result, i + 1));
        }
    }

    Err(ParseError::UnexpectedEof)
}

/// Calculate the canonical length of a varint encoding for a value.
fn canonical_varint_len(value: u64) -> usize {
    if value == 0 {
        return 1;
    }
    // Number of bits needed, divided by 7, rounded up
    let bits = 64 - value.leading_zeros() as usize;
    bits.div_ceil(7)
}

/// Encode a u64 value as a varint.
///
/// Always produces canonical (minimal) encoding.
#[cfg(test)]
pub fn encode_varint(mut value: u64) -> Vec<u8> {
    let mut result = Vec::with_capacity(MAX_VARINT_BYTES);

    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;

        if value != 0 {
            byte |= 0x80; // Set continuation bit
        }

        result.push(byte);

        if value == 0 {
            break;
        }
    }

    result
}

/// Encode a u64 value as a varint into an existing buffer.
pub fn encode_varint_into(mut value: u64, buf: &mut Vec<u8>) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;

        if value != 0 {
            byte |= 0x80;
        }

        buf.push(byte);

        if value == 0 {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_zero() {
        let encoded = encode_varint(0);
        assert_eq!(encoded, vec![0x00]);
        let (decoded, len) = decode_varint(&encoded).unwrap();
        assert_eq!(decoded, 0);
        assert_eq!(len, 1);
    }

    #[test]
    fn test_encode_decode_small() {
        let encoded = encode_varint(1);
        assert_eq!(encoded, vec![0x01]);
        let (decoded, len) = decode_varint(&encoded).unwrap();
        assert_eq!(decoded, 1);
        assert_eq!(len, 1);

        let encoded = encode_varint(127);
        assert_eq!(encoded, vec![0x7F]);
        let (decoded, len) = decode_varint(&encoded).unwrap();
        assert_eq!(decoded, 127);
        assert_eq!(len, 1);
    }

    #[test]
    fn test_encode_decode_two_bytes() {
        let encoded = encode_varint(128);
        assert_eq!(encoded, vec![0x80, 0x01]);
        let (decoded, len) = decode_varint(&encoded).unwrap();
        assert_eq!(decoded, 128);
        assert_eq!(len, 2);

        let encoded = encode_varint(300);
        assert_eq!(encoded, vec![0xAC, 0x02]);
        let (decoded, len) = decode_varint(&encoded).unwrap();
        assert_eq!(decoded, 300);
        assert_eq!(len, 2);
    }

    #[test]
    fn test_encode_decode_large() {
        let encoded = encode_varint(u64::MAX);
        assert_eq!(encoded.len(), 10);
        let (decoded, len) = decode_varint(&encoded).unwrap();
        assert_eq!(decoded, u64::MAX);
        assert_eq!(len, 10);
    }

    #[test]
    fn test_non_canonical_accepted() {
        // Non-canonical encoding of 1: extra continuation byte
        let non_canonical = vec![0x81, 0x00];
        let (decoded, len) = decode_varint(&non_canonical).unwrap();
        assert_eq!(decoded, 1);
        assert_eq!(len, 2);
    }

    #[test]
    fn test_unexpected_eof() {
        // Continuation bit set but no more bytes
        let incomplete = vec![0x80];
        assert_eq!(decode_varint(&incomplete), Err(ParseError::UnexpectedEof));
    }

    #[test]
    fn test_too_many_bytes() {
        // More than 10 bytes
        let too_long = vec![0x80; 11];
        assert_eq!(decode_varint(&too_long), Err(ParseError::InvalidVarint));
    }

    #[test]
    fn test_overflow() {
        // 10th byte with value > 1 would overflow u64
        let overflow = vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x02];
        assert_eq!(decode_varint(&overflow), Err(ParseError::InvalidVarint));
    }
}
