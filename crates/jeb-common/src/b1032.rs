//! B1032: Human-readable integer encoding optimized for common values.
//!
//! Encodes non-negative integers as short alphanumeric strings. Common values
//! (0-9999) encode as familiar decimal; larger values use base32 for density.
//!
//! The encoding is bijective and unambiguous: every integer maps to exactly one
//! canonical token, and every valid token decodes to exactly one integer.
//!
//! # Examples
//!
//! ```
//! use jeb_common::b1032::{encode, decode};
//!
//! // Small values are decimal
//! assert_eq!(encode(42), "42");
//! assert_eq!(encode(9999), "9999");
//!
//! // Larger values use base32 (at least 4 chars, always contains a letter)
//! assert_eq!(encode(10000), "000A");
//! assert_eq!(encode(1_000_000), "UGI0");
//!
//! // Decoding is case-insensitive
//! assert_eq!(decode("000a").unwrap(), 10000);
//! ```
//!
//! # Design
//!
//! The tricky part: base32 tokens like "1234" look like decimal. To avoid
//! ambiguity, tokens ≤4 chars that are all-digits are always decoded as decimal.
//! The encoder skips base32 values that would produce such tokens, ensuring
//! integers ≥10000 always encode to strings containing at least one letter.
//!
//! Base32 alphabet: `0123456789ABCDEFGHIJKLMNOPQRSTUV` (digits 0-9, letters A-V).

use std::fmt;

/// Base32 alphabet used by B1032.
const ALPHABET: &[u8; 32] = b"0123456789ABCDEFGHIJKLMNOPQRSTUV";

/// B = 32^4 = 1,048,576
const B: u64 = 32_u64.pow(4);

/// C = 304,426 (base32 "999A") - start of uninterrupted identity run.
const C: u64 = 304_426;

/// Error type for B1032 operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Empty input string.
    EmptyString,
    /// Invalid character in base32 string.
    InvalidCharacter(char),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::EmptyString => write!(f, "empty string"),
            Error::InvalidCharacter(c) => write!(f, "invalid base32 character: {:?}", c),
        }
    }
}

impl std::error::Error for Error {}

/// Convert a character to its base32 value.
fn char_to_value(c: char) -> Option<u64> {
    match c {
        '0'..='9' => Some((c as u64) - ('0' as u64)),
        'A'..='V' => Some((c as u64) - ('A' as u64) + 10),
        'a'..='v' => Some((c as u64) - ('a' as u64) + 10),
        _ => None,
    }
}

/// Standard base32 encoding (no padding).
fn to_base32_raw(mut n: u64) -> String {
    if n == 0 {
        return "0".to_string();
    }
    let mut out = Vec::new();
    while n > 0 {
        out.push(ALPHABET[(n % 32) as usize]);
        n /= 32;
    }
    out.reverse();
    String::from_utf8(out).unwrap()
}

/// Base32 encode n and left-pad with '0' to at least 4 chars.
fn to_base32_min4(n: u64) -> String {
    let raw = to_base32_raw(n);
    if raw.len() >= 4 {
        raw
    } else {
        format!("{:0>4}", raw)
    }
}

/// Parse base32 token using the alphabet.
fn from_base32(s: &str) -> Result<u64, Error> {
    if s.is_empty() {
        return Err(Error::EmptyString);
    }
    let mut n: u64 = 0;
    for ch in s.chars() {
        let d = char_to_value(ch).ok_or(Error::InvalidCharacter(ch))?;
        n = n * 32 + d;
    }
    Ok(n)
}

/// Return the 4 base32 digits of v in [0, 32^4) as integers 0..31.
fn digits4(mut v: u64) -> [u64; 4] {
    let mut ds = [0u64; 4];
    for (i, p) in [32_u64.pow(3), 32_u64.pow(2), 32, 1].iter().enumerate() {
        ds[i] = v / p;
        v %= p;
    }
    ds
}

/// Count of values v' in [0..=v] whose 4-digit padded base32 digits are all <
/// 10 (i.e., token would be digit-only).
fn bad_leq(v: u64) -> u64 {
    let ds = digits4(v);
    let mut tight: u64 = 1;
    let mut loose: u64 = 0;

    for lim in ds {
        let mut nt: u64 = 0;
        let mut nl: u64 = 0;

        // From tight: choose x in 0..9 with x <= lim
        for x in 0..10_u64 {
            if x > lim {
                break;
            }
            if x == lim {
                nt += tight;
            } else {
                nl += tight;
            }
        }
        // From loose: choose any x in 0..9
        nl += loose * 10;

        tight = nt;
        loose = nl;
    }
    tight + loose
}

/// Count of 'good' values in [0..=v], where good means padded-4 has at least
/// one letter.
fn good_leq(v: u64) -> u64 {
    (v + 1) - bad_leq(v)
}

/// 0-based rank of good value v among all good values in [0, 32^4).
fn rank_good(v: u64) -> u64 {
    good_leq(v) - 1
}

/// Return the k-th good value (0-based) in [0, 32^4), by binary search.
fn unrank_good(k: u64) -> u64 {
    debug_assert!(k < B - 10_000, "k out of range");

    let mut lo: u64 = 0;
    let mut hi: u64 = B - 1;
    let target = k + 1; // want good_leq(v) >= target

    while lo < hi {
        let mid = (lo + hi) / 2;
        if good_leq(mid) >= target {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }
    lo
}

/// Encode non-negative integer n to a human-friendly B1032 token.
pub fn encode(n: u64) -> String {
    if n <= 9999 {
        return n.to_string();
    }

    // Fast path: from C onward, it's just base32 (min width 4).
    if n >= C {
        return to_base32_min4(n);
    }

    // Tricky region: allocate the (n-10000)-th good v in [0, 32^4)
    let v = unrank_good(n - 10_000);
    to_base32_min4(v)
}

/// Decode B1032 token back to the original integer.
pub fn decode(t: &str) -> Result<u64, Error> {
    if t.is_empty() {
        return Err(Error::EmptyString);
    }

    // Strip leading zeros first.
    let t = t.trim_start_matches('0');
    let t = if t.is_empty() { "0" } else { t };

    let all_digits = t.chars().all(|c| c.is_ascii_digit());

    // All digits, ≤4 chars → decimal
    if all_digits && t.len() <= 4 {
        return Ok(t.parse::<u64>().unwrap());
    }

    // All digits, ≥5 chars → plain base32 (always >= C)
    // Has letters → base32 (transitional if v < C, plain otherwise)
    let v = from_base32(t)?;

    // Fast path: from C onward, decoding is identical to base32 parsing.
    if v >= C {
        return Ok(v);
    }

    // Otherwise, it's in the tricky zone: map good-value rank back to n.
    Ok(10_000 + rank_good(v))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_small() {
        for n in 0..=9999_u64 {
            let tok = encode(n);
            let back = decode(&tok).unwrap();
            assert_eq!(n, back, "failed for n={}", n);
        }
    }

    #[test]
    fn test_roundtrip_boundary() {
        // Test around key boundaries
        let test_values = [
            0,
            7,
            42,
            9998,
            9999,
            10000,
            10001,
            304423,
            304424,
            304425,
            304426,
            304427,
            B - 2,
            B - 1,
            B,
            B + 1,
        ];

        for &n in &test_values {
            let tok = encode(n);
            let back = decode(&tok).unwrap();
            assert_eq!(n, back, "failed for n={}, tok={}", n, tok);
        }
    }

    #[test]
    fn test_specific_encodings() {
        assert_eq!(encode(9999), "9999");
        assert_eq!(encode(10000), "000A");
        assert_eq!(encode(304425), "998V");
        assert_eq!(encode(304426), "999A"); // C
        assert_eq!(encode(304427), "999B");
        assert_eq!(encode(1048575), "VVVV"); // B - 1
        assert_eq!(encode(1048576), "10000"); // B
    }

    #[test]
    fn test_specific_decodings() {
        assert_eq!(decode("9999").unwrap(), 9999);
        assert_eq!(decode("000A").unwrap(), 10000);
        assert_eq!(decode("999A").unwrap(), 304426);
        assert_eq!(decode("VVVV").unwrap(), 1048575);
        assert_eq!(decode("10000").unwrap(), 1048576);
    }

    #[test]
    fn test_case_insensitive_decode() {
        assert_eq!(decode("000a").unwrap(), 10000);
        assert_eq!(decode("999a").unwrap(), 304426);
        assert_eq!(decode("vvvv").unwrap(), 1048575);
    }

    #[test]
    fn test_decimal_tokens_no_leading_zeros() {
        // Canonical decimal has no leading zeros
        assert_eq!(encode(0), "0");
        assert_eq!(encode(7), "7");
        assert_eq!(encode(42), "42");
        assert_eq!(encode(100), "100");
    }

    #[test]
    fn test_decode_with_leading_zeros() {
        // Decode should handle leading zeros in decimal range
        assert_eq!(decode("0007").unwrap(), 7);
        assert_eq!(decode("0042").unwrap(), 42);
        assert_eq!(decode("0100").unwrap(), 100);
    }

    #[test]
    fn test_error_empty_string() {
        assert_eq!(decode(""), Err(Error::EmptyString));
    }

    #[test]
    fn test_error_invalid_char() {
        assert_eq!(decode("WXYZ"), Err(Error::InvalidCharacter('W')));
        assert_eq!(decode("!@#$"), Err(Error::InvalidCharacter('!')));
    }

    #[test]
    fn test_roundtrip_tricky_region() {
        // Test the entire tricky region (10000 to C-1)
        for n in 10000..C {
            let tok = encode(n);
            let back = decode(&tok).unwrap();
            assert_eq!(n, back, "failed for n={}, tok={}", n, tok);
        }
    }

    #[test]
    fn test_roundtrip_post_c_region() {
        // Test a range after C
        for n in C..(C + 10000) {
            let tok = encode(n);
            let back = decode(&tok).unwrap();
            assert_eq!(n, back, "failed for n={}, tok={}", n, tok);
        }
    }

    #[test]
    fn test_roundtrip_large_values() {
        // Test some large values
        let test_values = [
            B,
            B + 1,
            B + 100,
            B + 10000,
            B * 2,
            B * 10,
            B * 100,
            u64::MAX / 2,
            u64::MAX - 1,
        ];

        for &n in &test_values {
            let tok = encode(n);
            let back = decode(&tok).unwrap();
            assert_eq!(n, back, "failed for n={}, tok={}", n, tok);
        }
    }

    #[test]
    fn test_encoded_tokens_not_ambiguous() {
        // Verify that tokens in the tricky region are not all-digit 4-char strings
        for n in 10000..C {
            let tok = encode(n);
            if tok.len() == 4 {
                assert!(
                    !tok.chars().all(|c| c.is_ascii_digit()),
                    "n={} produced ambiguous token: {}",
                    n,
                    tok
                );
            }
        }
    }

    #[test]
    fn test_base32_raw() {
        assert_eq!(to_base32_raw(0), "0");
        assert_eq!(to_base32_raw(31), "V");
        assert_eq!(to_base32_raw(32), "10");
        assert_eq!(to_base32_raw(1023), "VV");
        assert_eq!(to_base32_raw(1024), "100");
    }

    #[test]
    fn test_digits4() {
        assert_eq!(digits4(0), [0, 0, 0, 0]);
        assert_eq!(digits4(1), [0, 0, 0, 1]);
        assert_eq!(digits4(32), [0, 0, 1, 0]);
        assert_eq!(digits4(32 * 32), [0, 1, 0, 0]);
        assert_eq!(digits4(32 * 32 * 32), [1, 0, 0, 0]);
        assert_eq!(digits4(B - 1), [31, 31, 31, 31]);
    }

    #[test]
    fn test_bad_leq() {
        // There are exactly 10^4 = 10000 bad values in [0, B)
        assert_eq!(bad_leq(B - 1), 10000);

        // "9999" in base32 = 9*32^3 + 9*32^2 + 9*32 + 9 = 304425
        // This is the last "bad" value (all digits, 4 chars)
        assert_eq!(bad_leq(304425), 10000);
    }

    #[test]
    fn test_good_count() {
        // Total good values in [0, B) = B - 10000
        assert_eq!(good_leq(B - 1), B - 10000);
    }

    #[test]
    fn test_extra_leading_zeros_with_letters() {
        // Base32 tokens with letters: extra leading zeros work
        assert_eq!(decode("000A").unwrap(), 10000);
        assert_eq!(decode("0000A").unwrap(), 10000);
        assert_eq!(decode("00000A").unwrap(), 10000);
    }

    #[test]
    fn test_leading_zeros_stripped() {
        // Leading zeros are stripped first, then length/content determines format
        // "00007" → "7" → decimal
        assert_eq!(decode("00007").unwrap(), 7);
        assert_eq!(decode("000042").unwrap(), 42);
        assert_eq!(decode("0000000000000000007").unwrap(), 7);

        // "0000A" → "A" → base32 (transitional, since A=10 < C)
        assert_eq!(decode("0000A").unwrap(), 10000);

        // All zeros → "0" → decimal 0
        assert_eq!(decode("0000").unwrap(), 0);
        assert_eq!(decode("00000000").unwrap(), 0);
    }
}
