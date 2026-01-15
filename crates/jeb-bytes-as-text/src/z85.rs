/// The Z85 alphabet string.
pub const Z85_CHARS: &[u8; 85] =
    b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";

/// Lookup table for decoding Z85 characters.
/// Maps ASCII bytes to their value (0-84) or 0xFF if invalid.
pub static Z85_DECODE_TABLE: [u8; 256] = {
    let mut table = [0xFF; 256];
    let chars = Z85_CHARS;
    let mut i = 0;
    while i < 85 {
        table[chars[i] as usize] = i as u8;
        i += 1;
    }
    table
};

/// Encodes a block of 4 bytes into 5 Z85 characters.
/// Panics if `output` len is < 5.
pub fn encode_block_4(input: &[u8; 4], output: &mut [u8]) {
    let value = u32::from_be_bytes(*input) as u64;
    let mut divisor = 52_200_625; // 85^4
    let mut v = value;

    for i in 0..5 {
        let idx = (v / divisor) as usize;
        output[i] = Z85_CHARS[idx];
        v %= divisor;
        divisor /= 85;
    }
}

/// Decodes a block of 5 Z85 characters into 4 bytes.
/// Returns None if any character is invalid or overflow occurs.
pub fn decode_block_5(input: &[u8; 5], output: &mut [u8]) -> Option<()> {
    let mut value: u64 = 0;
    let mut multiplier = 52_200_625; // 85^4

    for &b in input {
        let digit = Z85_DECODE_TABLE[b as usize];
        if digit == 0xFF {
            return None;
        }
        value += (digit as u64) * multiplier;
        multiplier /= 85;
    }

    if value > u32::MAX as u64 {
        return None;
    }

    output.copy_from_slice(&(value as u32).to_be_bytes());
    Some(())
}

/// Encodes binary data into a Z85 string.
/// Handles partial blocks by padding with zeros and truncating the output.
pub fn encode(data: &[u8]) -> String {
    if data.is_empty() {
        return String::new();
    }

    // Estimate capacity: 5 chars for every 4 bytes (ceil).
    let len = data.len();
    let cap = (len * 5 + 3) / 4;
    let mut output = Vec::with_capacity(cap);

    let mut chunks = data.chunks_exact(4);
    let mut buf = [0u8; 5];

    // Encode full blocks
    for chunk in chunks.by_ref() {
        encode_block_4(chunk.try_into().unwrap(), &mut buf);
        output.extend_from_slice(&buf);
    }

    // Handle partial block
    let remainder = chunks.remainder();
    if !remainder.is_empty() {
        let mut padded = [0u8; 4];
        padded[..remainder.len()].copy_from_slice(remainder);
        encode_block_4(&padded, &mut buf);

        // Calculate partial length 'c' using ceiling division
        // c = ceil(b * 5 / 4)
        let b = remainder.len();
        let c = (b * 5 + 3) / 4;
        output.extend_from_slice(&buf[..c]);
    }

    String::from_utf8(output).expect("Z85 charset is valid UTF-8")
}

/// Decodes a Z85 string into binary data.
/// Handles partial blocks by padding with '0' and truncating the output.
pub fn decode(data: &str) -> Result<Vec<u8>, String> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    let input = data.as_bytes();
    let len = input.len();

    // Z85 maps 5 chars -> 4 bytes.
    // Estimated capacity b = floor(c * 4 / 5)
    let cap = (len * 4) / 5;
    let mut output = Vec::with_capacity(cap);

    let mut chunks = input.chunks_exact(5);
    let mut buf = [0u8; 4];

    // Decode full blocks
    for (i, chunk) in chunks.by_ref().enumerate() {
        decode_block_5(chunk.try_into().unwrap(), &mut buf)
            .ok_or_else(|| format!("Invalid Z85 block at index {}", i * 5))?;
        output.extend_from_slice(&buf);
    }

    // Handle partial block
    let remainder = chunks.remainder();
    if !remainder.is_empty() {
        let c = remainder.len();
        // Check for valid partial length: 1 char is impossible (min 2 chars for 1 byte)
        // Valid mappings:
        // 1 byte -> 2 chars
        // 2 bytes -> 3 chars
        // 3 bytes -> 4 chars
        if c == 1 {
            return Err("Invalid Z85 length: partial block cannot be 1 character".to_string());
        }

        let mut padded = [0u8; 5]; // Padding char is '0' (value 0)
        padded[..c].copy_from_slice(remainder);

        // Pad with '0' (the character for value 0)
        // Wait, analysis shows we must pad with the MAX value (84) to round up
        // correctly.
        for i in c..5 {
            padded[i] = Z85_CHARS[84];
        }

        decode_block_5(&padded, &mut buf)
            .ok_or_else(|| format!("Invalid Z85 partial block at end"))?;

        // Calculate partial byte length 'b' using floor division
        // b = floor(c * 4 / 5)
        let b = (c * 4) / 5;
        output.extend_from_slice(&buf[..b]);
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_world() {
        // "Hello World" is 11 bytes.
        // [He ll o ] [Wo rl d \0]
        // Blocks: "Hello " (6 bytes? No space is 0x20)
        // "Hello World" bytes: 48 65 6c 6c 6f 20 57 6f 72 6c 64
        // Chunk 1: "Hell" (48 65 6c 6c) -> nm=QN
        // Chunk 2: "o Wo" (6f 20 57 6f) -> 75?2
        // Chunk 3: "rld" (72 6c 64) -> Partial 3 bytes

        let input = b"Hello World";
        let encoded = encode(input);

        // ZMQ spec example: eight octets -> 86 % (6F 63 74 65 74 73)
        // "86 %" -> [0x86, 0x20, 0x25, 0x20] -> "JTKVS"

        // Let's rely on roundtrip for correctness verification rather than manual calc
        let decoded = decode(&encoded).expect("Should decode");
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_partials() {
        // 1 byte -> 2 chars
        let data = b"A";
        let enc = encode(data);
        assert_eq!(enc.len(), 2);
        assert_eq!(decode(&enc).unwrap(), data);

        // 2 bytes -> 3 chars
        let data = b"AB";
        let enc = encode(data);
        assert_eq!(enc.len(), 3);
        assert_eq!(decode(&enc).unwrap(), data);

        // 3 bytes -> 4 chars
        let data = b"ABC";
        let enc = encode(data);
        assert_eq!(enc.len(), 4);
        assert_eq!(decode(&enc).unwrap(), data);
    }

    #[test]
    fn test_empty() {
        assert_eq!(encode(b""), "");
        assert_eq!(decode("").unwrap(), b"");
    }

    #[test]
    fn test_binary_edge_cases() {
        let all_zeros = vec![0u8; 100];
        let enc = encode(&all_zeros);
        let dec = decode(&enc).unwrap();
        assert_eq!(dec, all_zeros);
        // All zeros should be '0' characters
        assert!(enc.chars().all(|c| c == '0'));

        let all_ones = vec![0xFFu8; 4];
        let enc = encode(&all_ones);
        // 0xFFFFFFFF is max u32 -> "####" (84,84,84,84,84) ?? No.
        // 85^5 is 4,437,053,125. 2^32-1 is 4,294,967,295.
        // It's just the max value.
        let dec = decode(&enc).unwrap();
        assert_eq!(dec, all_ones);
    }

    #[test]
    fn test_invalid_length() {
        // 1 char is invalid
        assert!(decode("0").is_err());
        // 6 chars is valid (1 full block + 1 partial) -> actually wait
        // 5 chars = 4 bytes.
        // 6 chars = 5 + 1. The last 1 char is a partial block of length 1?
        // NO. A partial block must be at least 2 chars to encode 1 byte.
        // So a total length of 6 chars implies the last block is 1 char long.
        // This should fail.
        assert!(decode("000000").is_err());
    }
}
