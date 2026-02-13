// Z85 Encoding/Decoding Implementation
// =====================================
//
// Z85 is a binary-to-text encoding scheme defined by ZeroMQ (RFC 32).
// It encodes binary data into printable ASCII characters, similar to Base64
// but with a different alphabet and encoding ratio.
//
// Key Properties:
// - Alphabet: 85 printable ASCII characters (excluding characters that might
//   cause issues in various contexts like quotes, backslash, etc.)
// - Encoding ratio: 4 bytes -> 5 characters (80% efficiency vs 75% for Base64)
// - Big-endian byte order for the 4-byte blocks
//
// This implementation extends standard Z85 to support arbitrary-length input
// (not just multiples of 4 bytes) using a scheme similar to unpadded Base64:
// - 1 byte  -> 2 characters
// - 2 bytes -> 3 characters
// - 3 bytes -> 4 characters
// - 4 bytes -> 5 characters
//
// =============================================================================
// Raw Passthrough Extension (`,` escape)
// =============================================================================
//
// This implementation includes an extension to Z85 that allows raw passthrough
// of 4-byte blocks when ALL bytes are "safe" printable characters.
//
// ENCODING:
// - When a 4-byte block consists entirely of "safe" characters, the encoder
//   MAY output `,` followed by the 4 raw bytes (5 chars total) instead of
//   standard Z85 encoding.
// - The `,` character acts as an escape marker indicating raw passthrough.
// - Safe characters for encoding decisions: Z85 alphabet plus `,;|~_`
//   (total: 0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#,;|~_)
// - Standard Z85 encoding is always valid; the encoder SHOULD use `,` passthrough
//   when possible for better readability.
// - `,` can ONLY appear at position 0 of a 5-character block (block-aligned).
//
// DECODING:
// - When `,` is encountered at a block boundary (position 0 mod 5), the next
//   4 bytes are taken as literal output (raw passthrough).
// - The decoder does NOT validate that raw bytes are "safe" - it trusts the input.
// - Otherwise, standard Z85 decoding is applied.
//
// This extension is backward-compatible: any standard Z85 input decodes correctly,
// and extended output can be decoded by extended decoders.

/// The Z85 alphabet: 85 printable ASCII characters in a specific order.
/// Characters are chosen to be safe in most contexts (no quotes, backslash, etc.)
/// Index 0 = '0', Index 84 = '#'
const Z85_ALPHABET: &[u8; 85] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";

/// The raw passthrough escape character.
/// When this appears at position 0 of a 5-character block during decoding,
/// the following 4 characters are taken as literal bytes (no Z85 decoding).
const RAW_ESCAPE: u8 = b',';

/// Extended safe characters for raw passthrough encoding decisions.
/// These are the Z85 alphabet (85 chars) plus 5 additional safe characters: `,;|~_`
/// Total: 90 characters that are considered "safe" for raw passthrough.
///
/// A 4-byte block qualifies for raw passthrough encoding (`,XXXX` format)
/// only if ALL 4 bytes are in this safe set.
const SAFE_CHARS: &[u8; 90] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#,;|~_";

/// Lookup table for safe character detection during encoding.
/// For each byte 0-255, stores true if the byte is a safe character for raw passthrough.
const SAFE_CHAR_TABLE: [bool; 256] = build_safe_char_table();

/// Build the safe character lookup table at compile time.
const fn build_safe_char_table() -> [bool; 256] {
    let mut table = [false; 256];
    let mut i = 0usize;
    while i < 90 {
        table[SAFE_CHARS[i] as usize] = true;
        i += 1;
    }
    table
}

/// Lookup table for decoding: maps ASCII byte value -> Z85 digit value (0-84)
/// Invalid characters are marked with 0xFF
const Z85_DECODE_TABLE: [u8; 256] = build_decode_table();

/// Build the decode lookup table at compile time.
/// For each byte 0-255, stores either the Z85 digit value (0-84) or 0xFF if invalid.
const fn build_decode_table() -> [u8; 256] {
    let mut table = [0xFFu8; 256];
    let mut i = 0usize;
    while i < 85 {
        table[Z85_ALPHABET[i] as usize] = i as u8;
        i += 1;
    }
    table
}

/// Error type for Z85 decoding failures
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// Input contains a character not in the Z85 alphabet
    InvalidCharacter(u8),
    /// The decoded value overflows the expected byte count
    /// (e.g., 5 characters that decode to > 0xFFFFFFFF)
    Overflow,
    /// Input length is invalid (1 char or 6 chars - can't map to valid byte counts)
    InvalidLength,
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecodeError::InvalidCharacter(c) => {
                write!(f, "invalid character in Z85 input: 0x{:02X}", c)
            }
            DecodeError::Overflow => write!(f, "Z85 value overflow"),
            DecodeError::InvalidLength => write!(f, "invalid Z85 input length"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Encode arbitrary bytes into a Z85 string.
///
/// # Algorithm
///
/// For each 4-byte block:
/// 1. Interpret the 4 bytes as a big-endian u32
/// 2. Convert to base-85 by repeatedly dividing by 85
/// 3. Map each base-85 digit to the corresponding alphabet character
///
/// The division produces digits in reverse order (least significant first),
/// so we fill the output buffer from right to left.
///
/// For trailing bytes (1-3 bytes), we:
/// 1. Pad conceptually with zeros on the right to form a partial block
/// 2. Encode only the significant portion:
///    - 1 byte  (8 bits)  -> 2 chars (ceil(8 * 5/4) / 5 = 2 chars needed)
///    - 2 bytes (16 bits) -> 3 chars
///    - 3 bytes (24 bits) -> 4 chars
///
/// # Example
///
/// Input: [0x86, 0x4F, 0xD2, 0x6F]
/// As u32 (big-endian): 0x864FD26F = 2,252,698,223
///
/// Successive division by 85:
/// 2252698223 / 85 = 26502332 remainder 3  -> alphabet[3] = '3' (but this goes last)
/// ... (continue division)
///
pub fn encode(input: &[u8]) -> String {
    // Handle empty input
    if input.is_empty() {
        return String::new();
    }

    // With non-aligned passthrough, we build output incrementally because
    // the alignment can shift based on where we place passthrough sections.
    let mut output: Vec<u8> = Vec::new();

    let mut in_idx = 0;

    while in_idx < input.len() {
        let bytes_remaining = input.len() - in_idx;

        // First, check if we have at least 4 bytes for a potential passthrough
        if bytes_remaining >= 4 {
            // Check for block-aligned passthrough (simplest case)
            if is_block_safe_for_passthrough(&input[in_idx..]) {
                // Block-aligned passthrough: just output , + 4 bytes
                output.push(RAW_ESCAPE);
                output.push(input[in_idx]);
                output.push(input[in_idx + 1]);
                output.push(input[in_idx + 2]);
                output.push(input[in_idx + 3]);
                in_idx += 4;
                continue;
            }

            // NOTE: Non-aligned passthrough encoding is disabled for now.
            // The decoder supports it, but the encoder complexity for ensuring
            // correct round-trip encoding of the "after" block is significant.
            // Only block-aligned passthrough (where all 4 bytes align) is used.

            // Standard Z85 encoding
            let value = u32::from_be_bytes([
                input[in_idx],
                input[in_idx + 1],
                input[in_idx + 2],
                input[in_idx + 3],
            ]);

            // Convert to base-85
            let mut v = value;
            let mut digits = [0u8; 5];
            for i in (0..5).rev() {
                digits[i] = Z85_ALPHABET[(v % 85) as usize];
                v /= 85;
            }
            output.extend_from_slice(&digits);
            in_idx += 4;
        } else {
            // Trailing bytes (1-3): always use standard Z85 encoding
            let num_bytes = bytes_remaining;
            let num_chars = num_bytes + 1;

            let value = match num_bytes {
                1 => input[in_idx] as u32,
                2 => u16::from_be_bytes([input[in_idx], input[in_idx + 1]]) as u32,
                3 => {
                    ((input[in_idx] as u32) << 16)
                        | ((input[in_idx + 1] as u32) << 8)
                        | (input[in_idx + 2] as u32)
                }
                _ => unreachable!(),
            };

            // Encode partial block
            let mut v = value;
            let mut digits = vec![0u8; num_chars];
            for i in (0..num_chars).rev() {
                digits[i] = Z85_ALPHABET[(v % 85) as usize];
                v /= 85;
            }
            output.extend_from_slice(&digits);
            in_idx += num_bytes;
        }
    }

    // Convert to String (all characters are ASCII, so this is safe)
    String::from_utf8(output).expect("Z85 output should be valid UTF-8")
}

/// Check if a 4-byte block consists entirely of safe characters for raw passthrough.
///
/// A block qualifies for raw passthrough if ALL 4 bytes are in the SAFE_CHARS set
/// (Z85 alphabet plus `,;|~_`). This allows the encoder to output `,XXXX` format
/// instead of standard Z85 encoding, which can improve readability for text-like data.
#[inline]
fn is_block_safe_for_passthrough(block: &[u8]) -> bool {
    block.len() >= 4
        && SAFE_CHAR_TABLE[block[0] as usize]
        && SAFE_CHAR_TABLE[block[1] as usize]
        && SAFE_CHAR_TABLE[block[2] as usize]
        && SAFE_CHAR_TABLE[block[3] as usize]
}

/// Encode a full 32-bit value into exactly 5 Z85 characters.
/// Fills the slice from right to left with base-85 digits.
fn encode_block_to_slice(mut value: u32, output: &mut [u8]) {
    debug_assert!(output.len() == 5);

    // Fill from right to left (least significant digit first)
    for i in (0..5).rev() {
        output[i] = Z85_ALPHABET[(value % 85) as usize];
        value /= 85;
    }
}

/// Encode a partial value (from 1-3 bytes) into the appropriate number of Z85 characters.
///
/// The math: for n input bytes, we need n+1 output characters.
/// We encode as if the value represents the most significant bits of a larger number.
fn encode_partial_to_slice(mut value: u32, num_chars: usize, output: &mut [u8]) {
    debug_assert!(num_chars >= 2 && num_chars <= 4);
    debug_assert!(output.len() == num_chars);

    // Fill from right to left
    for i in (0..num_chars).rev() {
        output[i] = Z85_ALPHABET[(value % 85) as usize];
        value /= 85;
    }
}

// =============================================================================
// Non-Aligned Passthrough Encoding Support
// =============================================================================

/// Get the P high-order Z85 digits for a 32-bit value.
///
/// The Z85 encoding of a 32-bit value produces 5 digits. This returns the first P digits.
fn get_high_order_z85_digits(value: u32, p: usize) -> Vec<u8> {
    // Full Z85 encoding produces 5 digits
    let mut digits = vec![0u8; 5];
    let mut v = value;
    for i in (0..5).rev() {
        digits[i] = (v % 85) as u8;
        v /= 85;
    }
    // Return first P digits
    digits.truncate(p);
    digits
}

/// Compute canonical minimum for encoding (returns None on error instead of panicking).
fn compute_canonical_minimum_for_encoding(
    high_digits: &[u8],
    known_low_bytes: &[u8],
) -> Option<u32> {
    let p = high_digits.len();
    let num_known_bytes = known_low_bytes.len();

    let mut base: u64 = 0;
    for &digit in high_digits {
        base = base * 85 + digit as u64;
    }

    let power = 85u64.pow((5 - p) as u32);
    let range_start = base * power;
    let range_end = (base + 1) * power;

    if num_known_bytes == 0 {
        if range_start > u32::MAX as u64 {
            return None;
        }
        return Some(range_start as u32);
    }

    let mut known_part: u64 = 0;
    for &byte in known_low_bytes {
        known_part = (known_part << 8) | byte as u64;
    }

    let modulus: u64 = 1 << (num_known_bytes * 8);
    let start_remainder = range_start % modulus;

    let candidate = if start_remainder <= known_part {
        range_start - start_remainder + known_part
    } else {
        range_start - start_remainder + modulus + known_part
    };

    if candidate >= range_end || candidate > u32::MAX as u64 {
        return None;
    }

    Some(candidate as u32)
}

/// Check if a block value is the canonical minimum for given partial Z85 encoding.
#[allow(dead_code)]
fn is_canonical_minimum(block_value: u32, p: usize, known_low_bytes: &[u8]) -> bool {
    let high_digits = get_high_order_z85_digits(block_value, p);
    match compute_canonical_minimum_for_encoding(&high_digits, known_low_bytes) {
        Some(canonical_min) => block_value == canonical_min,
        None => false,
    }
}

/// Reconstruct the "after" block value from known high bytes and low Z85 digits.
///
/// After a non-aligned passthrough at position P, the "after" block has:
/// - P known high bytes from the passthrough
/// - (5-P) low-order Z85 digits from the input
///
/// This function finds the 32-bit value V such that:
/// - V's high P bytes equal known_high_bytes
/// - V mod 85^num_digits equals low_digits_value
fn reconstruct_after_block_value(
    known_high_bytes: &[u8],
    low_digits_value: u64,
    num_digits: usize,
) -> Result<u32, DecodeError> {
    let p = known_high_bytes.len();

    // Compute the known high value (big-endian)
    let mut known_high: u64 = 0;
    for &b in known_high_bytes {
        known_high = (known_high << 8) | b as u64;
    }

    // The full value V must satisfy:
    // - (V >> (8 * (4-P))) == known_high (high bytes match)
    // - V % 85^num_digits == low_digits_value (low digits match)

    let shift = 8 * (4 - p);
    let range_start = known_high << shift;
    let range_size: u64 = 1 << shift;

    let modulus = 85u64.pow(num_digits as u32);

    // Find smallest V >= range_start where V % modulus == low_digits_value
    let start_remainder = range_start % modulus;

    let candidate = if start_remainder <= low_digits_value {
        range_start - start_remainder + low_digits_value
    } else {
        range_start - start_remainder + modulus + low_digits_value
    };

    // Verify candidate is in range
    if candidate >= range_start + range_size {
        return Err(DecodeError::InvalidLength);
    }

    Ok(candidate as u32)
}

/// Decode a Z85 string back into bytes.
///
/// # Algorithm
///
/// For each 5-character block:
/// 1. Check if the first character is `,` (raw passthrough escape)
///    - If so, take the next 4 characters as literal bytes (no Z85 decoding)
///    - The decoder does NOT validate that raw bytes are "safe" - it trusts the input
/// 2. Otherwise, apply standard Z85 decoding:
///    - Map each character to its base-85 digit value (0-84)
///    - Accumulate: value = d0*85^4 + d1*85^3 + d2*85^2 + d3*85 + d4
///    - Convert the u32 value to 4 big-endian bytes
///
/// # Non-Aligned Passthrough (Advanced)
///
/// When `,` appears at position P (1-4) within a 5-char block, it interrupts
/// the Z85 encoding of surrounding blocks:
/// - First P chars are partial Z85 digits for the "before" block
/// - `,` + next 4 chars are raw passthrough bytes
/// - The "before" block is ambiguous: we compute all possible 32-bit values
///   and output the MINIMUM (canonical) value (big-endian interpretation)
///
/// For trailing characters (2-4 chars), we:
/// 1. Decode to get the partial value (standard Z85, no passthrough for partials)
/// 2. Extract only the appropriate number of bytes:
///    - 2 chars -> 1 byte
///    - 3 chars -> 2 bytes
///    - 4 chars -> 3 bytes
///
/// # Errors
///
/// - `InvalidCharacter`: Input contains a byte not in the Z85 alphabet
/// - `Overflow`: The accumulated value exceeds what can fit in the target bytes
/// - `InvalidLength`: Input length is 1 or 6 (mod 5), which can't map to valid byte counts
///
pub fn decode(input: &str) -> Result<Vec<u8>, DecodeError> {
    let input = input.as_bytes();

    // Handle empty input
    if input.is_empty() {
        return Ok(Vec::new());
    }

    // With non-aligned passthrough, we need to track additional state because
    // passthrough bytes overlap with the before/after blocks.
    let mut output = Vec::new();

    let mut in_idx = 0;

    // Track position within current Z85 block (0-4)
    let mut block_pos = 0usize;
    // Accumulated Z85 digits for current block
    let mut current_block_digits: Vec<u8> = Vec::with_capacity(5);
    // Known high bytes for current block (from passthrough of previous block)
    let mut known_high_bytes: Vec<u8> = Vec::new();

    while in_idx < input.len() {
        let byte = input[in_idx];

        if byte == RAW_ESCAPE {
            // Found a comma - this is a passthrough marker
            let p = block_pos; // Position within the 5-char block (0-4)

            // Ensure we have at least 4 more characters for the passthrough bytes
            if in_idx + 4 >= input.len() {
                return Err(DecodeError::InvalidLength);
            }

            // Extract the 4 passthrough bytes
            let pass_bytes = &input[in_idx + 1..in_idx + 5];

            if p == 0 {
                // Block-aligned passthrough: simple case, just output the 4 bytes
                output.extend_from_slice(pass_bytes);
                in_idx += 5; // Skip comma + 4 bytes
                // block_pos stays at 0, current_block_digits stays empty, known_high_bytes stays empty
            } else {
                // Non-aligned passthrough at position P (1-4)
                //
                // Structure:
                // - We have P high-order Z85 digits for "before" block
                // - pass_bytes[0..4-P] are the known low bytes of "before" block
                // - pass_bytes[4-P..4] are the known high bytes of "after" block
                //
                // The passthrough bytes OVERLAP with both blocks!

                let num_known_low_bytes = 4 - p;
                let known_low_bytes = &pass_bytes[0..num_known_low_bytes];

                // Compute the canonical minimum for the "before" block
                let before_value = compute_canonical_minimum(&current_block_digits, known_low_bytes)?;

                // Output the 4 bytes of the "before" block
                output.extend_from_slice(&before_value.to_be_bytes());

                // DO NOT output passthrough bytes separately - they overlap with before/after blocks!
                // Instead, set the known high bytes for the "after" block
                known_high_bytes = pass_bytes[num_known_low_bytes..].to_vec(); // Last P bytes

                // Reset block state for "after" block
                current_block_digits.clear();
                block_pos = 0;

                // Move past comma + 4 passthrough bytes
                in_idx += 5;
            }
        } else {
            // Regular Z85 character
            let digit = Z85_DECODE_TABLE[byte as usize];
            if digit == 0xFF {
                return Err(DecodeError::InvalidCharacter(byte));
            }

            current_block_digits.push(digit);
            block_pos += 1;
            in_idx += 1;

            // Check if we have enough digits to complete the current block
            // Normal case: 5 digits
            // After non-aligned passthrough: (5-P) digits where P = known_high_bytes.len()
            let needed_digits = 5 - known_high_bytes.len();

            if current_block_digits.len() == needed_digits {
                let value: u32;

                if known_high_bytes.is_empty() {
                    // Normal case: decode full 5-digit block
                    let mut v: u64 = 0;
                    for &d in &current_block_digits {
                        v = v * 85 + d as u64;
                    }

                    // Check for overflow
                    if v > u32::MAX as u64 {
                        return Err(DecodeError::Overflow);
                    }
                    value = v as u32;
                } else {
                    // After non-aligned passthrough: reconstruct block from known high bytes + low digits
                    let mut low_digits_value: u64 = 0;
                    for &d in &current_block_digits {
                        low_digits_value = low_digits_value * 85 + d as u64;
                    }

                    value = reconstruct_after_block_value(&known_high_bytes, low_digits_value, needed_digits)?;

                    // Clear known_high_bytes for next block
                    known_high_bytes.clear();
                }

                // Output 4 bytes
                output.extend_from_slice(&value.to_be_bytes());

                // Reset for next block
                current_block_digits.clear();
                block_pos = 0;
            }
        }
    }

    // Handle trailing partial block (if any)
    if !current_block_digits.is_empty() {
        let num_chars = current_block_digits.len();

        // Invalid: 1 character doesn't map to a valid byte count
        if num_chars == 1 {
            return Err(DecodeError::InvalidLength);
        }

        // Decode partial block: 2 chars -> 1 byte, 3 chars -> 2 bytes, 4 chars -> 3 bytes
        let mut value: u64 = 0;
        for &d in &current_block_digits {
            value = value * 85 + d as u64;
        }

        let num_bytes = num_chars - 1;

        // Check overflow based on expected byte count
        match num_bytes {
            1 => {
                if value > 0xFF {
                    return Err(DecodeError::Overflow);
                }
                output.push(value as u8);
            }
            2 => {
                if value > 0xFFFF {
                    return Err(DecodeError::Overflow);
                }
                output.extend_from_slice(&(value as u16).to_be_bytes());
            }
            3 => {
                if value > 0xFFFFFF {
                    return Err(DecodeError::Overflow);
                }
                output.push((value >> 16) as u8);
                output.push((value >> 8) as u8);
                output.push(value as u8);
            }
            _ => unreachable!(),
        }
    }

    Ok(output)
}

/// Decode a full 5-character block into a u32.
/// Returns error if any character is invalid or if the value overflows u32.
fn decode_block(block: &[u8]) -> Result<u32, DecodeError> {
    debug_assert!(block.len() == 5);

    let mut value: u64 = 0;

    // Accumulate: value = d0*85^4 + d1*85^3 + d2*85^2 + d3*85 + d4
    // We use u64 to detect overflow before converting to u32
    for &byte in block {
        let digit = Z85_DECODE_TABLE[byte as usize];
        if digit == 0xFF {
            return Err(DecodeError::InvalidCharacter(byte));
        }
        value = value * 85 + digit as u64;
    }

    // Check for overflow (max valid Z85 5-char value is 85^5 - 1 = 4,437,053,124)
    // But we need it to fit in u32 (max 4,294,967,295 = 0xFFFFFFFF)
    if value > u32::MAX as u64 {
        return Err(DecodeError::Overflow);
    }

    Ok(value as u32)
}

/// Decode a partial block (2, 3, or 4 characters) into a u32.
/// The value represents a partial number (1, 2, or 3 bytes worth).
fn decode_partial_block(block: &[u8]) -> Result<u32, DecodeError> {
    debug_assert!(block.len() >= 2 && block.len() <= 4);

    let mut value: u64 = 0;

    for &byte in block {
        let digit = Z85_DECODE_TABLE[byte as usize];
        if digit == 0xFF {
            return Err(DecodeError::InvalidCharacter(byte));
        }
        value = value * 85 + digit as u64;
    }

    // No overflow check here - we check in the caller based on expected byte count
    Ok(value as u32)
}

// =============================================================================
// Non-Aligned Passthrough Support
// =============================================================================
//
// When `,` appears at position P (1-4) within a 5-char block, the Z85 encoding
// is interrupted. The "before" block becomes ambiguous because we only have
// P high-order Z85 digits and the raw passthrough bytes provide (4-P) known
// low-order input bytes.
//
// To resolve ambiguity deterministically, we compute ALL possible 32-bit values
// that could have produced the observed partial Z85 + known bytes, then select
// the MINIMUM value (canonical). This ensures:
// - Decoding is deterministic and unambiguous
// - Encoding can check if actual value equals canonical minimum before using passthrough

/// Compute the canonical (minimum) 32-bit value for an ambiguous "before" block.
///
/// Given P high-order Z85 digits and (4-P) known low-order input bytes (from passthrough),
/// find the minimum 32-bit value V such that:
/// 1. V's Z85 encoding starts with the given P digits
/// 2. V's big-endian bytes end with the given (4-P) known bytes
///
/// # Algorithm
///
/// - P Z85 digits define a range: [base, base + 85^(5-P)) where base = digits * 85^(5-P)
/// - Within this range, find values where low bytes match the known passthrough bytes
/// - Return the minimum such value
///
/// # Arguments
///
/// * `high_digits` - Slice of P Z85 digit values (0-84)
/// * `known_low_bytes` - Slice of (4-P) known low-order bytes from passthrough
///
/// # Returns
///
/// The canonical minimum 32-bit value, or error if no valid value exists
fn compute_canonical_minimum(
    high_digits: &[u8],
    known_low_bytes: &[u8],
) -> Result<u32, DecodeError> {
    let p = high_digits.len();
    let num_known_bytes = known_low_bytes.len(); // Should be 4 - p

    // Compute the base value from high Z85 digits
    // base = d0 * 85^(5-1) + d1 * 85^(5-2) + ... + d(P-1) * 85^(5-P)
    let mut base: u64 = 0;
    for &digit in high_digits {
        base = base * 85 + digit as u64;
    }

    // The range of possible values is [base * 85^(5-P), (base+1) * 85^(5-P))
    let power = 85u64.pow((5 - p) as u32);
    let range_start = base * power;
    let range_end = (base + 1) * power;

    // Now we need to find values in [range_start, range_end) whose big-endian bytes
    // end with known_low_bytes
    if num_known_bytes == 0 {
        // P = 4: No constraint from known bytes, just return range_start
        if range_start > u32::MAX as u64 {
            return Err(DecodeError::Overflow);
        }
        return Ok(range_start as u32);
    }

    // Construct the constraint from the known bytes
    let mut known_part: u64 = 0;
    for &byte in known_low_bytes {
        known_part = (known_part << 8) | byte as u64;
    }

    // The mask for the known bytes (low num_known_bytes bytes)
    let modulus: u64 = 1 << (num_known_bytes * 8);

    // Find smallest V >= range_start where V % modulus === known_part
    let start_remainder = range_start % modulus;

    let candidate = if start_remainder <= known_part {
        range_start - start_remainder + known_part
    } else {
        range_start - start_remainder + modulus + known_part
    };

    // Verify candidate is in range and fits in u32
    if candidate >= range_end {
        return Err(DecodeError::InvalidLength); // No valid value exists
    }
    if candidate > u32::MAX as u64 {
        return Err(DecodeError::Overflow);
    }

    Ok(candidate as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty() {
        assert_eq!(encode(&[]), "");
        assert_eq!(decode("").unwrap(), vec![]);
    }

    #[test]
    fn test_zeros_4bytes() {
        assert_eq!(encode(&[0, 0, 0, 0]), "00000");
        assert_eq!(decode("00000").unwrap(), vec![0, 0, 0, 0]);
    }

    #[test]
    fn test_max_4bytes() {
        assert_eq!(encode(&[0xff, 0xff, 0xff, 0xff]), "%nSc0");
        assert_eq!(decode("%nSc0").unwrap(), vec![0xff, 0xff, 0xff, 0xff]);
    }

    #[test]
    fn test_one_byte() {
        assert_eq!(encode(&[0x00]), "00");
        assert_eq!(decode("00").unwrap(), vec![0x00]);

        // 0xFF = 255 = 3*85 + 0, so encodes to "30"
        assert_eq!(encode(&[0xff]), "30");
        assert_eq!(decode("30").unwrap(), vec![0xff]);
    }

    #[test]
    fn test_roundtrip() {
        let test_cases: &[&[u8]] = &[
            &[],
            &[0],
            &[0, 0],
            &[0, 0, 0],
            &[0, 0, 0, 0],
            &[0, 0, 0, 0, 0],
            &[0xff],
            &[0xff, 0xff],
            &[0xff, 0xff, 0xff],
            &[0xff, 0xff, 0xff, 0xff],
            &[1, 2, 3, 4],
            &[1, 2, 3, 4, 5, 6, 7, 8],
        ];

        for input in test_cases {
            let encoded = encode(input);
            let decoded = decode(&encoded).unwrap();
            assert_eq!(decoded, *input, "roundtrip failed for {:?}", input);
        }
    }

    #[test]
    fn test_invalid_character() {
        // Use valid length strings (5 chars) with invalid characters
        assert!(matches!(decode("hel\"o"), Err(DecodeError::InvalidCharacter(b'"'))));
        assert!(matches!(decode("hel o"), Err(DecodeError::InvalidCharacter(b' '))));
    }

    #[test]
    fn test_invalid_length() {
        // Length 1 is invalid
        assert!(matches!(decode("0"), Err(DecodeError::InvalidLength)));
        // Length 6 is invalid (would be 1 mod 5)
        assert!(matches!(decode("000000"), Err(DecodeError::InvalidLength)));
    }

    #[test]
    fn test_overflow() {
        // "#####" = 84*85^4 + 84*85^3 + 84*85^2 + 84*85 + 84 = 4,437,053,124 > 0xFFFFFFFF
        assert!(matches!(decode("#####"), Err(DecodeError::Overflow)));

        // "##" for 1 byte: 84*85 + 84 = 7224 > 255
        assert!(matches!(decode("##"), Err(DecodeError::Overflow)));
    }

    // =========================================================================
    // Tests for raw passthrough extension (`,` escape)
    // =========================================================================

    #[test]
    fn test_raw_passthrough_encode() {
        // "test" (4 ASCII chars) should be encoded with passthrough
        let input = b"test";
        let encoded = encode(input);
        assert_eq!(encoded, ",test", "4 safe chars should use passthrough");
    }

    #[test]
    fn test_raw_passthrough_decode() {
        // ",test" should decode to "test"
        let decoded = decode(",test").unwrap();
        assert_eq!(decoded, b"test");
    }

    #[test]
    fn test_raw_passthrough_roundtrip() {
        // Various safe character combinations
        let test_cases: &[&[u8]] = &[
            b"test",
            b"abcd",
            b"ABCD",
            b"1234",
            b".-:+",
            b",;|~",  // Extended safe chars
        ];

        for input in test_cases {
            let encoded = encode(input);
            // Should use passthrough (starts with ,)
            assert!(encoded.starts_with(','), "Expected passthrough for {:?}", input);
            let decoded = decode(&encoded).unwrap();
            assert_eq!(decoded, *input, "roundtrip failed for {:?}", input);
        }
    }

    #[test]
    fn test_mixed_passthrough_and_z85() {
        // Mix of safe and non-safe blocks
        // First 4 bytes: 0x00 0x00 0x00 0x00 (not safe - contains null bytes)
        // Next 4 bytes: "test" (safe)
        let input = [0x00, 0x00, 0x00, 0x00, b't', b'e', b's', b't'];
        let encoded = encode(&input);
        // Should be "00000,test" - first block Z85, second passthrough
        assert_eq!(encoded, "00000,test");

        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_no_passthrough_for_unsafe_bytes() {
        // Bytes that are not in safe chars set
        let input = [0x00, 0x01, 0x02, 0x03];
        let encoded = encode(&input);
        // Should NOT start with `,` - not safe for passthrough
        assert!(!encoded.starts_with(','), "Non-safe bytes should use Z85");

        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_passthrough_decode_does_not_validate() {
        // Decoder should accept ANY bytes after `,`, not just safe ones
        // This is important: decoder trusts the input
        let encoded = ",\x00\x01\x02\x03";  // Not actually safe chars
        let decoded = decode(encoded).unwrap();
        assert_eq!(decoded, [0x00, 0x01, 0x02, 0x03]);
    }

    #[test]
    fn test_passthrough_with_trailing_bytes() {
        // "test" + null byte (5 bytes)
        // First 4 bytes: "test" (safe, passthrough)
        // Trailing 1 byte: 0x00 (Z85 encoded)
        let input = [b't', b'e', b's', b't', 0x00];
        let encoded = encode(&input);
        assert_eq!(encoded, ",test00");  // passthrough + 1-byte Z85

        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, input);
    }

    #[test]
    fn test_non_aligned_passthrough_position_1() {
        // Non-aligned passthrough: comma at position 1
        // "A,BCDE" - 'A' is 1 Z85 digit, then passthrough 'BCDE'
        // The "before" block has 1 Z85 digit (A=36) and 3 known bytes (BCD = 0x42,0x43,0x44)
        //
        // Passthrough bytes overlap with before/after blocks:
        // - 'BCD' are the last 3 bytes of "before" block
        // - 'E' is the first byte of "after" block (need 4 more Z85 digits)
        //
        // Since there are no chars after passthrough, only "before" block is output.
        let result = decode("A,BCDE").unwrap();
        assert_eq!(result.len(), 4);
        // Before block: canonical minimum with high digit 36, low bytes 0x42,0x43,0x44
        assert_eq!(&result[..], &[0x70, 0x42, 0x43, 0x44]);
    }

    #[test]
    fn test_non_aligned_passthrough_incomplete() {
        // Comma at position 4 without enough bytes after should fail
        let result = decode("ABCD,");
        assert!(matches!(result, Err(DecodeError::InvalidLength)),
                "Expected InvalidLength for incomplete passthrough");
    }

    #[test]
    fn test_non_aligned_passthrough_position_4() {
        // Non-aligned passthrough: comma at position 4
        // "ABCD,efgh" - 4 Z85 digits, then passthrough 'efgh'
        //
        // Passthrough bytes overlap:
        // - 0 bytes are the last (4-4)=0 bytes of "before" block
        // - 'efgh' (4 bytes) are the first 4 bytes of "after" block
        //
        // Since P=4, we need 5-4=1 more Z85 digit for "after" block, but there are none.
        // So only "before" block is output (4 bytes).
        let result = decode("ABCD,efgh").unwrap();
        assert_eq!(result.len(), 4);
        // Before block: canonical minimum with 4 digits (A=36,B=37,C=38,D=39)
        assert_eq!(&result[..], &[0x71, 0x61, 0x9E, 0x8E]);
    }
}
