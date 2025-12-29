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
//! use jeb_common::b1032::{
//!     from_b1032,
//!     to_b1032,
//! };
//!
//! // Small values are decimal
//! assert_eq!(to_b1032(42_u64), "42");
//! assert_eq!(to_b1032(9999_u32), "9999");
//!
//! // Larger values use base32 (at least 4 chars, always contains a letter)
//! assert_eq!(to_b1032(10000_u64), "000A");
//! assert_eq!(to_b1032(1_000_000_u64), "UGI0");
//!
//! // Decoding is case-insensitive and generic over return type
//! assert_eq!(from_b1032::<u64>("000a").unwrap(), 10000);
//! assert_eq!(from_b1032::<u32>("42").unwrap(), 42);
//! ```
//!
//! # Design
//!
//! The tricky part: base32 tokens like "1234" look like decimal. To avoid
//! ambiguity, tokens ≤4 chars that are all-digits are always decoded as
//! decimal. The encoder skips base32 values that would produce such tokens,
//! ensuring integers ≥10000 always encode to strings containing at least one
//! letter.
//!
//! Base32 alphabet: `0123456789ABCDEFGHIJKLMNOPQRSTUV` (digits 0-9, letters
//! A-V).

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
    /// Invalid character in input string.
    InvalidCharacter(char),
    /// Value overflows the target type.
    Overflow,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::EmptyString => write!(f, "empty string"),
            Error::InvalidCharacter(c) => write!(f, "invalid character: {:?}", c),
            Error::Overflow => write!(f, "value overflows target type"),
        }
    }
}

impl std::error::Error for Error {}

/// Trait for types that can be encoded/decoded using B1032.
pub trait B1032: Sized {
    /// Encode this value to a B1032 string.
    fn to_b1032(self) -> String;

    /// Decode a B1032 string to this type.
    fn from_b1032(s: &str) -> Result<Self, Error>;
}

/// Encode a value to B1032. Generic over any type implementing `B1032`.
pub fn to_b1032<T: B1032>(value: T) -> String {
    value.to_b1032()
}

/// Decode a B1032 string. Generic over return type.
pub fn from_b1032<T: B1032>(s: &str) -> Result<T, Error> {
    T::from_b1032(s)
}

// =============================================================================
// Internal implementation
// =============================================================================

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
        n = n.checked_mul(32).ok_or(Error::Overflow)?;
        n = n.checked_add(d).ok_or(Error::Overflow)?;
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

/// Core encode implementation (works on u64).
fn encode_u64(n: u64) -> String {
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

/// Core decode implementation (returns u64).
fn decode_u64(t: &str) -> Result<u64, Error> {
    if t.is_empty() {
        return Err(Error::EmptyString);
    }

    // Check for any invalid characters first (including whitespace, etc.)
    for ch in t.chars() {
        if !matches!(ch, '0'..='9' | 'A'..='V' | 'a'..='v') {
            return Err(Error::InvalidCharacter(ch));
        }
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

// =============================================================================
// Trait implementations
// =============================================================================

impl B1032 for u64 {
    fn to_b1032(self) -> String {
        encode_u64(self)
    }

    fn from_b1032(s: &str) -> Result<Self, Error> {
        decode_u64(s)
    }
}

impl B1032 for u32 {
    fn to_b1032(self) -> String {
        encode_u64(self as u64)
    }

    fn from_b1032(s: &str) -> Result<Self, Error> {
        let v = decode_u64(s)?;
        v.try_into().map_err(|_| Error::Overflow)
    }
}

impl B1032 for u16 {
    fn to_b1032(self) -> String {
        encode_u64(self as u64)
    }

    fn from_b1032(s: &str) -> Result<Self, Error> {
        let v = decode_u64(s)?;
        v.try_into().map_err(|_| Error::Overflow)
    }
}

impl B1032 for u8 {
    fn to_b1032(self) -> String {
        encode_u64(self as u64)
    }

    fn from_b1032(s: &str) -> Result<Self, Error> {
        let v = decode_u64(s)?;
        v.try_into().map_err(|_| Error::Overflow)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper: assert encode and decode roundtrip
    fn assert_roundtrip_u64(n: u64) {
        let tok = to_b1032(n);
        let back = from_b1032(&tok).unwrap();
        assert_eq!(n, back, "roundtrip failed for n={}, tok={}", n, tok);
    }

    fn assert_roundtrip<T: B1032 + Copy + PartialEq + std::fmt::Debug>(n: T, expected_tok: &str) {
        let tok = to_b1032(n);
        assert_eq!(tok, expected_tok, "encode mismatch for {:?}", n);
        let back: T = from_b1032(&tok).unwrap();
        assert_eq!(n, back, "roundtrip failed for {:?}, tok={}", n, tok);
    }

    // =========================================================================
    // Basic roundtrip tests
    // =========================================================================

    #[test]
    fn test_roundtrip_small() {
        for n in 0..=9999_u64 {
            assert_roundtrip_u64(n);
        }
    }

    #[test]
    fn test_roundtrip_boundary() {
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
            assert_roundtrip_u64(n);
        }
    }

    #[test]
    fn test_roundtrip_tricky_region() {
        for n in 10000..C {
            assert_roundtrip_u64(n);
        }
    }

    #[test]
    fn test_roundtrip_post_c_region() {
        for n in C..(C + 10000) {
            assert_roundtrip_u64(n);
        }
    }

    #[test]
    fn test_roundtrip_large_values() {
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
            u64::MAX,
        ];

        for &n in &test_values {
            assert_roundtrip_u64(n);
        }
    }

    // =========================================================================
    // Specific encoding/decoding tests
    // =========================================================================

    #[test]
    fn test_specific_encodings() {
        assert_eq!(to_b1032(9999), "9999");
        assert_eq!(to_b1032(10000), "000A");
        assert_eq!(to_b1032(304425), "998V");
        assert_eq!(to_b1032(304426), "999A"); // C
        assert_eq!(to_b1032(304427), "999B");
        assert_eq!(to_b1032(1048575), "VVVV"); // B - 1
        assert_eq!(to_b1032(1048576), "10000"); // B
    }

    #[test]
    fn test_specific_decodings() {
        assert_eq!(from_b1032("9999").unwrap(), 9999);
        assert_eq!(from_b1032("000A").unwrap(), 10000);
        assert_eq!(from_b1032("999A").unwrap(), 304426);
        assert_eq!(from_b1032("VVVV").unwrap(), 1048575);
        assert_eq!(from_b1032("10000").unwrap(), 1048576);
    }

    #[test]
    fn test_decimal_tokens_no_leading_zeros() {
        assert_eq!(to_b1032(0), "0");
        assert_eq!(to_b1032(7), "7");
        assert_eq!(to_b1032(42), "42");
        assert_eq!(to_b1032(100), "100");
    }

    #[test]
    fn test_encoded_tokens_not_ambiguous() {
        for n in 10000..C {
            let tok = to_b1032(n);
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

    // =========================================================================
    // Case insensitivity and leading zeros
    // =========================================================================

    #[test]
    fn test_case_insensitive_decode() {
        assert_eq!(from_b1032("000a").unwrap(), 10000);
        assert_eq!(from_b1032("999a").unwrap(), 304426);
        assert_eq!(from_b1032("vvvv").unwrap(), 1048575);
    }

    #[test]
    fn test_mixed_case_decode() {
        assert_eq!(from_b1032("VvVv").unwrap(), 1048575);
        assert_eq!(from_b1032("aB").unwrap(), from_b1032("AB").unwrap());
        assert_eq!(from_b1032("Ab").unwrap(), from_b1032("AB").unwrap());
    }

    #[test]
    fn test_decode_with_leading_zeros() {
        assert_eq!(from_b1032("0007").unwrap(), 7);
        assert_eq!(from_b1032("0042").unwrap(), 42);
        assert_eq!(from_b1032("0100").unwrap(), 100);
    }

    #[test]
    fn test_leading_zeros_stripped() {
        // "00007" → "7" → decimal
        assert_eq!(from_b1032("00007").unwrap(), 7);
        assert_eq!(from_b1032("000042").unwrap(), 42);
        assert_eq!(from_b1032("0000000000000000007").unwrap(), 7);

        // "0000A" → "A" → base32 (transitional, since A=10 < C)
        assert_eq!(from_b1032("0000A").unwrap(), 10000);

        // All zeros → "0" → decimal 0
        assert_eq!(from_b1032("0000").unwrap(), 0);
        assert_eq!(from_b1032("00000000").unwrap(), 0);
    }

    #[test]
    fn test_extra_leading_zeros_with_letters() {
        assert_eq!(from_b1032("000A").unwrap(), 10000);
        assert_eq!(from_b1032("0000A").unwrap(), 10000);
        assert_eq!(from_b1032("00000A").unwrap(), 10000);
    }

    // =========================================================================
    // Error cases
    // =========================================================================

    #[test]
    fn test_error_empty_string() {
        assert_eq!(from_b1032(""), Err(Error::EmptyString));
    }

    #[test]
    fn test_error_invalid_characters() {
        // Letters beyond V
        assert_eq!(from_b1032("W"), Err(Error::InvalidCharacter('W')));
        assert_eq!(from_b1032("w"), Err(Error::InvalidCharacter('w')));
        assert_eq!(from_b1032("X"), Err(Error::InvalidCharacter('X')));
        assert_eq!(from_b1032("Z"), Err(Error::InvalidCharacter('Z')));

        // Whitespace
        assert_eq!(from_b1032(" "), Err(Error::InvalidCharacter(' ')));
        assert_eq!(from_b1032("123 "), Err(Error::InvalidCharacter(' ')));
        assert_eq!(from_b1032(" 123"), Err(Error::InvalidCharacter(' ')));
        assert_eq!(from_b1032("12 34"), Err(Error::InvalidCharacter(' ')));
        assert_eq!(from_b1032("\t"), Err(Error::InvalidCharacter('\t')));
        assert_eq!(from_b1032("\n"), Err(Error::InvalidCharacter('\n')));
        assert_eq!(from_b1032("123\n"), Err(Error::InvalidCharacter('\n')));

        // Underscores and other punctuation
        assert_eq!(from_b1032("1_000"), Err(Error::InvalidCharacter('_')));
        assert_eq!(from_b1032("1,000"), Err(Error::InvalidCharacter(',')));
        assert_eq!(from_b1032("1.5"), Err(Error::InvalidCharacter('.')));
        assert_eq!(from_b1032("-1"), Err(Error::InvalidCharacter('-')));
        assert_eq!(from_b1032("+1"), Err(Error::InvalidCharacter('+')));

        // Special characters
        assert_eq!(from_b1032("!@#$"), Err(Error::InvalidCharacter('!')));
    }

    // =========================================================================
    // Generic type tests
    // =========================================================================

    #[test]
    fn test_u8_roundtrip() {
        for n in 0..=u8::MAX {
            let tok = to_b1032(n);
            let back: u8 = from_b1032(&tok).unwrap();
            assert_eq!(n, back);
        }
    }

    #[test]
    fn test_u8_specific() {
        assert_roundtrip(0_u8, "0");
        assert_roundtrip(1_u8, "1");
        assert_roundtrip(255_u8, "255");
    }

    #[test]
    fn test_u8_overflow() {
        // 256 doesn't fit in u8
        assert_eq!(from_b1032::<u8>("256"), Err(Error::Overflow));
        assert_eq!(from_b1032::<u8>("1000"), Err(Error::Overflow));
        assert_eq!(from_b1032::<u8>("000A"), Err(Error::Overflow)); // 10000
    }

    #[test]
    fn test_u16_roundtrip() {
        // Test boundaries
        for n in [0_u16, 1, 9999, 10000, 65534, 65535] {
            let tok = to_b1032(n);
            let back: u16 = from_b1032(&tok).unwrap();
            assert_eq!(n, back, "u16 roundtrip failed for {}", n);
        }
    }

    #[test]
    fn test_u16_specific() {
        assert_roundtrip(0_u16, "0");
        assert_roundtrip(9999_u16, "9999");
        assert_roundtrip(10000_u16, "000A");
        assert_roundtrip(65535_u16, "1O5V"); // u16::MAX (in tricky region < C)
    }

    #[test]
    fn test_u16_overflow() {
        // 65536 doesn't fit in u16
        assert_eq!(from_b1032::<u16>("65536"), Err(Error::Overflow));
        assert_eq!(from_b1032::<u16>("VVVV"), Err(Error::Overflow)); // 1048575
    }

    #[test]
    fn test_u32_roundtrip() {
        let test_values = [
            0_u32,
            1,
            9999,
            10000,
            304426,
            1048576,
            u32::MAX - 1,
            u32::MAX,
        ];
        for n in test_values {
            let tok = to_b1032(n);
            let back: u32 = from_b1032(&tok).unwrap();
            assert_eq!(n, back, "u32 roundtrip failed for {}", n);
        }
    }

    #[test]
    fn test_u32_specific() {
        assert_roundtrip(0_u32, "0");
        assert_roundtrip(u32::MAX, "3VVVVVV"); // 4294967295
    }

    #[test]
    fn test_u32_overflow() {
        // u32::MAX + 1 = 4294967296
        assert_eq!(from_b1032::<u32>("4000000"), Err(Error::Overflow));
        // u64::MAX encoded
        let max_tok = to_b1032(u64::MAX);
        assert_eq!(from_b1032::<u32>(&max_tok), Err(Error::Overflow));
    }

    #[test]
    fn test_u64_boundaries() {
        // Around u64::MAX
        assert_roundtrip_u64(u64::MAX - 2);
        assert_roundtrip_u64(u64::MAX - 1);
        assert_roundtrip_u64(u64::MAX);
    }

    // =========================================================================
    // Internal function tests
    // =========================================================================

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
        assert_eq!(bad_leq(B - 1), 10000);
        assert_eq!(bad_leq(304425), 10000);
    }

    #[test]
    fn test_good_count() {
        assert_eq!(good_leq(B - 1), B - 10000);
    }
}
