//! Implementation of the `Arbitrary` trait for fuzz testing.

use crate::Value;
use arbitrary::{Arbitrary, Unstructured};
use std::collections::HashSet;
use std::sync::{LazyLock, Mutex};

/// Maximum length for generated static strings.
/// This bounds memory usage per unique string.
const MAX_STRING_LENGTH: usize = 64;

/// Global interner for static strings.
/// Uses deduplication to avoid leaking duplicate strings.
static INTERNED_STRINGS: LazyLock<Mutex<HashSet<&'static str>>> =
    LazyLock::new(|| Mutex::new(HashSet::new()));

/// Intern a string, returning a `&'static str`.
///
/// This function deduplicates strings to minimize memory usage during fuzzing.
/// Strings are leaked (intentionally) to produce `&'static str` references,
/// but the deduplication ensures each unique string is only leaked once.
fn intern_string(s: &str) -> &'static str {
    // Truncate to max length to bound memory per string
    let s = if s.len() > MAX_STRING_LENGTH {
        // Find a valid UTF-8 boundary for truncation
        let mut end = MAX_STRING_LENGTH;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        &s[..end]
    } else {
        s
    };

    let mut set = INTERNED_STRINGS.lock().unwrap();

    // Check if we already have this string interned
    if let Some(&existing) = set.get(s) {
        return existing;
    }

    // Leak the string to get a &'static str
    let leaked: &'static str = Box::leak(s.to_owned().into_boxed_str());
    set.insert(leaked);
    leaked
}

/// Generate an arbitrary static string.
///
/// This generates a random string and interns it to produce a `&'static str`.
/// The interner deduplicates strings to minimize memory usage.
fn arbitrary_static_str(u: &mut Unstructured<'_>) -> arbitrary::Result<&'static str> {
    // Generate an arbitrary string (will be length-limited by intern_string)
    let s: String = u.arbitrary()?;
    Ok(intern_string(&s))
}

/// Generate arbitrary nested values with bounded depth.
fn arbitrary_value_with_depth(u: &mut Unstructured<'_>, depth: usize) -> arbitrary::Result<Value> {
    // Limit recursion depth to prevent stack overflow
    if depth == 0 {
        // At max depth, only generate non-recursive variants
        let variant = u.int_in_range(0..=17)?;
        return Ok(match variant {
            0 => Value::Bool(u.arbitrary()?),
            1 => Value::I8(u.arbitrary()?),
            2 => Value::I16(u.arbitrary()?),
            3 => Value::I32(u.arbitrary()?),
            4 => Value::I64(u.arbitrary()?),
            5 => Value::I128(u.arbitrary()?),
            6 => Value::U8(u.arbitrary()?),
            7 => Value::U16(u.arbitrary()?),
            8 => Value::U32(u.arbitrary()?),
            9 => Value::U64(u.arbitrary()?),
            10 => Value::U128(u.arbitrary()?),
            11 => Value::F32(u.arbitrary()?),
            12 => Value::F64(u.arbitrary()?),
            13 => Value::Char(u.arbitrary()?),
            14 => Value::String(u.arbitrary()?),
            15 => Value::Bytes(u.arbitrary()?),
            16 => Value::None,
            17 => Value::Unit,
            _ => unreachable!(),
        });
    }

    // Choose from all 29 variants
    let variant = u.int_in_range(0..=28)?;
    Ok(match variant {
        // Primitives (14)
        0 => Value::Bool(u.arbitrary()?),
        1 => Value::I8(u.arbitrary()?),
        2 => Value::I16(u.arbitrary()?),
        3 => Value::I32(u.arbitrary()?),
        4 => Value::I64(u.arbitrary()?),
        5 => Value::I128(u.arbitrary()?),
        6 => Value::U8(u.arbitrary()?),
        7 => Value::U16(u.arbitrary()?),
        8 => Value::U32(u.arbitrary()?),
        9 => Value::U64(u.arbitrary()?),
        10 => Value::U128(u.arbitrary()?),
        11 => Value::F32(u.arbitrary()?),
        12 => Value::F64(u.arbitrary()?),
        13 => Value::Char(u.arbitrary()?),

        // String and Bytes (2)
        14 => Value::String(u.arbitrary()?),
        15 => Value::Bytes(u.arbitrary()?),

        // Option (2)
        16 => Value::None,
        17 => Value::Some(Box::new(arbitrary_value_with_depth(u, depth - 1)?)),

        // Unit types (2)
        18 => Value::Unit,
        19 => Value::UnitStruct {
            name: arbitrary_static_str(u)?,
        },

        // Newtype (2)
        20 => Value::NewtypeStruct {
            name: arbitrary_static_str(u)?,
            value: Box::new(arbitrary_value_with_depth(u, depth - 1)?),
        },
        21 => Value::NewtypeVariant {
            enum_name: arbitrary_static_str(u)?,
            variant_index: u.arbitrary()?,
            variant: arbitrary_static_str(u)?,
            value: Box::new(arbitrary_value_with_depth(u, depth - 1)?),
        },

        // Sequences (4)
        22 => {
            let len = u.int_in_range(0..=4)?;
            let mut values = Vec::with_capacity(len);
            for _ in 0..len {
                values.push(arbitrary_value_with_depth(u, depth - 1)?);
            }
            Value::Seq(values)
        }
        23 => {
            let len = u.int_in_range(0..=4)?;
            let mut values = Vec::with_capacity(len);
            for _ in 0..len {
                values.push(arbitrary_value_with_depth(u, depth - 1)?);
            }
            Value::Tuple(values)
        }
        24 => {
            let len = u.int_in_range(0..=4)?;
            let mut fields = Vec::with_capacity(len);
            for _ in 0..len {
                fields.push(arbitrary_value_with_depth(u, depth - 1)?);
            }
            Value::TupleStruct {
                name: arbitrary_static_str(u)?,
                fields,
            }
        }
        25 => {
            let len = u.int_in_range(0..=4)?;
            let mut fields = Vec::with_capacity(len);
            for _ in 0..len {
                fields.push(arbitrary_value_with_depth(u, depth - 1)?);
            }
            Value::TupleVariant {
                enum_name: arbitrary_static_str(u)?,
                variant_index: u.arbitrary()?,
                variant: arbitrary_static_str(u)?,
                fields,
            }
        }

        // Maps and Structs (3)
        26 => {
            let len = u.int_in_range(0..=4)?;
            let mut entries = Vec::with_capacity(len);
            for _ in 0..len {
                let key = arbitrary_value_with_depth(u, depth - 1)?;
                let value = arbitrary_value_with_depth(u, depth - 1)?;
                entries.push((key, value));
            }
            Value::Map(entries)
        }
        27 => {
            let len = u.int_in_range(0..=4)?;
            let mut fields = Vec::with_capacity(len);
            for _ in 0..len {
                let name = arbitrary_static_str(u)?;
                let value = arbitrary_value_with_depth(u, depth - 1)?;
                fields.push((name, value));
            }
            Value::Struct {
                name: arbitrary_static_str(u)?,
                fields,
            }
        }
        28 => {
            let len = u.int_in_range(0..=4)?;
            let mut fields = Vec::with_capacity(len);
            for _ in 0..len {
                let name = arbitrary_static_str(u)?;
                let value = arbitrary_value_with_depth(u, depth - 1)?;
                fields.push((name, value));
            }
            Value::StructVariant {
                enum_name: arbitrary_static_str(u)?,
                variant_index: u.arbitrary()?,
                variant: arbitrary_static_str(u)?,
                fields,
            }
        }

        // Unit Variant (already covered by primitives section above)
        _ => Value::UnitVariant {
            enum_name: arbitrary_static_str(u)?,
            variant_index: u.arbitrary()?,
            variant: arbitrary_static_str(u)?,
        },
    })
}

impl<'a> Arbitrary<'a> for Value {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        // Use depth limit of 4 to keep generated values reasonable
        arbitrary_value_with_depth(u, 4)
    }

    fn size_hint(_depth: usize) -> (usize, Option<usize>) {
        // At minimum we need 1 byte to choose a variant
        // Upper bound is hard to determine due to recursion
        (1, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arbitrary_value_generation() {
        // Test that we can generate values from arbitrary bytes
        let data = [0u8; 256];
        let mut u = Unstructured::new(&data);
        let value = Value::arbitrary(&mut u);
        assert!(value.is_ok());
    }

    #[test]
    fn test_arbitrary_multiple_values() {
        // Generate multiple values to exercise different variants
        let data: Vec<u8> = (0..=255).collect();
        let mut u = Unstructured::new(&data);
        let mut count = 0;
        while let Ok(_value) = Value::arbitrary(&mut u) {
            count += 1;
            if count >= 10 {
                break;
            }
        }
        assert!(count > 0);
    }

    #[test]
    fn test_intern_string_deduplication() {
        let s1 = intern_string("test_dedup");
        let s2 = intern_string("test_dedup");
        // Same string should return same pointer
        assert!(std::ptr::eq(s1, s2));
    }

    #[test]
    fn test_intern_string_truncation() {
        let long_string = "a".repeat(100);
        let interned = intern_string(&long_string);
        assert!(interned.len() <= MAX_STRING_LENGTH);
    }

    #[test]
    fn test_intern_string_utf8_boundary() {
        // Test with a string that has multi-byte UTF-8 characters
        // Each emoji is 4 bytes, so 20 emojis = 80 bytes
        let emojis = "😀".repeat(20);
        let interned = intern_string(&emojis);
        // Should be truncated but still valid UTF-8
        assert!(interned.len() <= MAX_STRING_LENGTH);
        // Should not panic on iteration (proves valid UTF-8)
        for _ in interned.chars() {}
    }

    #[test]
    fn test_arbitrary_static_str() {
        let data: Vec<u8> = (0..=255).collect();
        let mut u = Unstructured::new(&data);
        let s = arbitrary_static_str(&mut u);
        assert!(s.is_ok());
    }
}
