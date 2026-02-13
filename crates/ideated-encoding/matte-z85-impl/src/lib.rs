//! Extended Z85 encoding with raw passthrough support.
//!
//! Implements position-invariant Z85 with mid-block boundary support.

use std::fmt;

const Z85_CHARS: &[u8; 85] = b"0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ.-:+=^!/*?&<>()[]{}@%$#";
const ESCAPE_CHARS: &[u8; 5] = b"_~|,;";

// Escape char meanings:
// '_' = block-aligned raw section
// '~' = 1 byte into block (1 leading char emitted)
// '|' = 2 bytes into block (2 leading chars emitted)
// ',' = 3 bytes into block (3 leading chars emitted)
// ';' = reserved

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EncodeError {
    InvalidInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    InvalidCharacter(u8),
    InvalidEscape,
    TruncatedInput,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DecodeError::InvalidCharacter(c) => write!(f, "Invalid character: {}", c),
            DecodeError::InvalidEscape => write!(f, "Invalid escape sequence"),
            DecodeError::TruncatedInput => write!(f, "Truncated input"),
        }
    }
}

impl std::error::Error for DecodeError {}

/// Build decode table for Z85 alphabet
fn build_decode_table() -> [u8; 256] {
    let mut table = [255u8; 256];
    for (i, &c) in Z85_CHARS.iter().enumerate() {
        table[c as usize] = i as u8;
    }
    table
}

static DECODE_TABLE: std::sync::OnceLock<[u8; 256]> = std::sync::OnceLock::new();

fn decode_table() -> &'static [u8; 256] {
    DECODE_TABLE.get_or_init(build_decode_table)
}

/// Encode 4 bytes to 5 Z85 characters
fn encode_block(bytes: &[u8; 4]) -> [u8; 5] {
    let mut value = 0u32;
    for &b in bytes {
        value = value * 256 + b as u32;
    }
    
    let mut result = [0u8; 5];
    for i in (0..5).rev() {
        result[i] = Z85_CHARS[(value % 85) as usize];
        value /= 85;
    }
    result
}

/// Decode 5 Z85 characters to 4 bytes
fn decode_block(chars: &[u8]) -> Result<[u8; 4], DecodeError> {
    if chars.len() != 5 {
        return Err(DecodeError::TruncatedInput);
    }
    
    let table = decode_table();
    let mut value = 0u32;
    
    for &c in chars {
        let digit = table[c as usize];
        if digit == 255 {
            return Err(DecodeError::InvalidCharacter(c));
        }
        value = value * 85 + digit as u32;
    }
    
    let mut result = [0u8; 4];
    for i in (0..4).rev() {
        result[i] = (value % 256) as u8;
        value /= 256;
    }
    
    Ok(result)
}

/// Check if byte has stable leading character (b < 174)
fn has_stable_leading_char(b: u8) -> bool {
    b < 174
}

/// Compute partial encoding for 1-3 bytes at mid-block boundary
/// Returns (encoded_chars, is_stable)
fn encode_mid_block_entry(bytes: &[u8]) -> (Vec<u8>, bool) {
    assert!(bytes.len() > 0 && bytes.len() < 4);
    
    // Check stability: all bytes must be < 174
    let stable = bytes.iter().all(|&b| has_stable_leading_char(b));
    
    if !stable {
        return (Vec::new(), false);
    }
    
    // Encode as partial block (same as encode_partial)
    // This gives unique reconstruction
    (encode_partial(bytes), true)
}

/// Check if mid-block entry + raw would violate position invariant (P1)
/// Returns true if the approach is valid (won't exceed standard Z85 length)
fn check_midblock_budget(leading_bytes: &[u8], raw_len: usize, total_len: usize) -> bool {
    // Mid-block cost: partial_chars + escape + length + raw_bytes
    let partial_chars = leading_bytes.len() + 1; // K bytes → K+1 chars
    let midblock_cost = partial_chars + 1 + 1 + raw_len;
    
    // Standard Z85 cost for the same total bytes
    let standard_cost = (total_len * 5 + 3) / 4;
    
    // P1 requirement: output ≤ standard Z85
    midblock_cost <= standard_cost
}

/// Encode 1-3 bytes as partial Z85 block
fn encode_partial(bytes: &[u8]) -> Vec<u8> {
    assert!(bytes.len() > 0 && bytes.len() < 4);
    
    // Build value from bytes (big-endian)
    let mut value = 0u32;
    for &b in bytes {
        value = value * 256 + b as u32;
    }
    
    // Calculate how many base-85 digits we need
    let num_chars = bytes.len() + 1;
    let mut result = vec![0u8; num_chars];
    
    // Encode to base-85 (LSB first)
    for i in (0..num_chars).rev() {
        result[i] = Z85_CHARS[(value % 85) as usize];
        value /= 85;
    }
    
    result
}

/// Decode partial Z85 block (2-4 chars for 1-3 bytes)
fn decode_partial(chars: &[u8]) -> Result<Vec<u8>, DecodeError> {
    if chars.len() < 2 || chars.len() > 4 {
        return Err(DecodeError::TruncatedInput);
    }
    
    let table = decode_table();
    let mut value = 0u32;
    
    for &c in chars {
        let digit = table[c as usize];
        if digit == 255 {
            return Err(DecodeError::InvalidCharacter(c));
        }
        value = value * 85 + digit as u32;
    }
    
    // Extract bytes (big-endian)
    let num_bytes = chars.len() - 1;
    let mut result = vec![0u8; num_bytes];
    
    for i in (0..num_bytes).rev() {
        result[i] = (value % 256) as u8;
        value /= 256;
    }
    
    Ok(result)
}

/// Check if we can do opportunistic exit at this position
/// (bytes would encode same with zero-padding)
fn can_exit_opportunistic(bytes: &[u8]) -> bool {
    if bytes.is_empty() || bytes.len() >= 4 {
        return false;
    }
    
    // Encode actual bytes
    let actual = encode_partial(bytes);
    
    // Encode with explicit zeros
    let mut with_zeros = bytes.to_vec();
    with_zeros.resize(4, 0);
    let mut padded = [0u8; 4];
    padded.copy_from_slice(&with_zeros);
    let zero_encoded = encode_block(&padded);
    
    // Check if they match for the chars we'd emit
    actual == &zero_encoded[..actual.len()]
}

/// Check if byte is raw-eligible (printable ASCII, not Z85, not escape)
fn is_raw_eligible(b: u8) -> bool {
    b >= 32 && b < 127 && decode_table()[b as usize] == 255 && !ESCAPE_CHARS.contains(&b)
}

/// Find next raw-eligible region of at least min_len bytes
fn find_raw_region(data: &[u8], start: usize, min_len: usize) -> Option<(usize, usize)> {
    let mut region_start = None;
    let mut i = start;
    
    while i < data.len() {
        if is_raw_eligible(data[i]) {
            if region_start.is_none() {
                region_start = Some(i);
            }
            i += 1;
        } else {
            if let Some(rs) = region_start {
                if i - rs >= min_len {
                    return Some((rs, i));
                }
            }
            region_start = None;
            i += 1;
        }
    }
    
    // Check final region
    if let Some(rs) = region_start {
        if data.len() - rs >= min_len {
            return Some((rs, data.len()));
        }
    }
    
    None
}

///Encoder with mid-block boundary support
pub fn encode(data: &[u8]) -> Vec<u8> {
    if data.is_empty() {
        return Vec::new();
    }
    
    let mut result = Vec::new();
    let mut pos = 0;
    
    'outer: while pos < data.len() {
        // Try to find raw-eligible region (min 5 bytes for net savings per §8 budget analysis)
        if let Some((raw_start, raw_end)) = find_raw_region(data, pos, 5) {
            // Encode up to raw region
            while pos < raw_start {
                let remaining_in_block = 4 - (pos % 4);
                let bytes_to_raw = raw_start - pos;
                
                if bytes_to_raw >= 4 || bytes_to_raw >= remaining_in_block && remaining_in_block == 4 {
                    // Full block
                    let block = [data[pos], data[pos + 1], data[pos + 2], data[pos + 3]];
                    result.extend_from_slice(&encode_block(&block));
                    pos += 4;
                } else if bytes_to_raw < remaining_in_block {
                    // Mid-block entry: emit partial encoding if stable AND budget permits
                    let boundary_bytes = &data[pos..raw_start];
                    let raw_len = raw_end - raw_start;
                    let total_len = (raw_end - pos).min(data.len() - pos);
                    let (partial_encoding, stable) = encode_mid_block_entry(boundary_bytes);
                    
                    // Check both stability and budget (P1: never exceed standard Z85 length)
                    if stable && check_midblock_budget(boundary_bytes, raw_len, total_len) {
                        // Emit partial encoding
                        result.extend_from_slice(&partial_encoding);
                        pos = raw_start;
                    } else {
                        // Can't do mid-block, encode full block and skip this raw region
                        // (will find raw region again on next iteration if it's still eligible)
                        let block = [
                            data[pos],
                            data.get(pos + 1).copied().unwrap_or(0),
                            data.get(pos + 2).copied().unwrap_or(0),
                            data.get(pos + 3).copied().unwrap_or(0),
                        ];
                        if pos + 4 <= data.len() {
                            result.extend_from_slice(&encode_block(&block));
                            pos += 4;
                        } else {
                            let partial = encode_partial(&data[pos..]);
                            result.extend_from_slice(&partial);
                            pos = data.len();
                        }
                        // Skip the raw section emission - will re-evaluate on next loop
                        continue 'outer;
                    }
                } else {
                    // Finish this block with partial encoding
                    let partial = encode_partial(&data[pos..raw_start]);
                    result.extend_from_slice(&partial);
                    pos = raw_start;
                }
            }
            
            // Emit raw section with appropriate escape char
            let raw_len = raw_end - raw_start;
            if raw_len > 255 {
                // Split into multiple sections if needed  
                let chunk_len = 255.min(raw_len);
                let bytes_into_block = raw_start % 4;
                let escape_char = ESCAPE_CHARS[bytes_into_block];
                result.push(escape_char);
                result.push(chunk_len as u8);
                result.extend_from_slice(&data[raw_start..raw_start + chunk_len]);
                pos = raw_start + chunk_len;
                continue;
            } else {
                let bytes_into_block = raw_start % 4;
                let escape_char = ESCAPE_CHARS[bytes_into_block];
                result.push(escape_char);
                result.push(raw_len as u8);
                result.extend_from_slice(&data[raw_start..raw_end]);
                pos = raw_end;
                
                // Mid-block exit: check if we can emit partial Z85 before next block boundary
                let bytes_into_block_at_exit = pos % 4;
                if bytes_into_block_at_exit > 0 && pos < data.len() {
                    let bytes_until_block = 4 - bytes_into_block_at_exit;
                    let remaining = data.len() - pos;
                    
                    // If remaining bytes exactly fill one or more blocks, skip opportunistic exit
                    // and let the main loop encode them as full blocks (more efficient)
                    if remaining >= 4 && remaining % 4 == 0 {
                        // Will be handled as full blocks by main loop
                    } else {
                        let bytes_to_check = bytes_until_block.min(remaining);
                        
                        if can_exit_opportunistic(&data[pos..pos + bytes_to_check]) {
                            let partial = encode_partial(&data[pos..pos + bytes_to_check]);
                            result.extend_from_slice(&partial);
                            pos += bytes_to_check;
                        }
                    }
                }
            }
        } else {
            // No more raw regions
            break;
        }
    }
    
    // Encode remaining data as Z85
    while pos < data.len() {
        if pos + 4 <= data.len() {
            let block = [data[pos], data[pos + 1], data[pos + 2], data[pos + 3]];
            result.extend_from_slice(&encode_block(&block));
            pos += 4;
        } else {
            let partial = encode_partial(&data[pos..]);
            result.extend_from_slice(&partial);
            break;
        }
    }
    
    result
}

/// Reconstruct bytes from partial encoding (for mid-block entry)
/// 
/// The partial encoding uses N+1 chars for N bytes (same as encode_partial).
/// This has unique reconstruction.
fn reconstruct_from_partial(partial_chars: &[u8]) -> Result<Vec<u8>, DecodeError> {
    // Just use decode_partial - it's the inverse of encode_partial
    decode_partial(partial_chars)
}

/// Decoder with mid-block boundary support
pub fn decode(data: &[u8]) -> Result<Vec<u8>, DecodeError> {
    if data.is_empty() {
        return Ok(Vec::new());
    }
    
    let mut result = Vec::new();
    let mut pos = 0;
    
    while pos < data.len() {
        // Look ahead for escape character
        let next_escape = data[pos..].iter().position(|&c| ESCAPE_CHARS.contains(&c));
        
        if let Some(escape_offset) = next_escape {
            let escape_pos = pos + escape_offset;
            
            // Decode any Z85 before the escape
            while pos < escape_pos {
                if pos + 5 <= escape_pos {
                    let block = decode_block(&data[pos..pos + 5])?;
                    result.extend_from_slice(&block);
                    pos += 5;
                } else {
                    // Partial encoding before escape
                    let partial_chars = &data[pos..escape_pos];
                    let reconstructed = reconstruct_from_partial(partial_chars)?;
                    result.extend_from_slice(&reconstructed);
                    pos = escape_pos;
                }
            }
            
            // Now handle the escape
            let _escape_idx = ESCAPE_CHARS.iter().position(|&c| c == data[pos]).unwrap();
            pos += 1; // Skip escape char
            
            if pos >= data.len() {
                return Err(DecodeError::TruncatedInput);
            }
            
            // Read length byte
            let raw_len = data[pos] as usize;
            pos += 1;
            
            if pos + raw_len > data.len() {
                return Err(DecodeError::TruncatedInput);
            }
            
            // Copy raw bytes
            result.extend_from_slice(&data[pos..pos + raw_len]);
            pos += raw_len;
        } else {
            // No more escapes, decode rest as Z85
            while pos < data.len() {
                if pos + 5 <= data.len() {
                    let block = decode_block(&data[pos..pos + 5])?;
                    result.extend_from_slice(&block);
                    pos += 5;
                } else {
                    // Partial block at end
                    let partial = decode_partial(&data[pos..])?;
                    result.extend_from_slice(&partial);
                    break;
                }
            }
            break;
        }
    }
    
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_block_roundtrip() {
        let data = b"Hell";
        let encoded = encode_block(&[b'H', b'e', b'l', b'l']);
        let decoded = decode_block(&encoded).unwrap();
        assert_eq!(&decoded, data);
    }
    
    #[test]
    fn test_full_roundtrip() {
        let data = b"Hello, World!";
        let encoded = encode(data);
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, data);
    }
    
    #[test]
    fn test_partial_blocks() {
        for len in 1..=3 {
            let data = &b"ABC"[..len];
            let encoded = encode(data);
            let decoded = decode(&encoded).unwrap();
            assert_eq!(decoded, data);
        }
    }
    
    #[test]
    fn test_empty() {
        let encoded = encode(b"");
        assert_eq!(encoded, b"");
        let decoded = decode(&encoded).unwrap();
        assert_eq!(decoded, b"");
    }
}
