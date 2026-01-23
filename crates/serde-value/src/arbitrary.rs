//! Implementation of the `Arbitrary` trait for fuzz testing.

use crate::intern::{intern_string, intern_string_owned};
use crate::Value;
use arbitrary::{Arbitrary, Unstructured};

/// Maximum length for generated static strings.
const MAX_STRING_LENGTH: usize = 64;

/// Intern a string, returning a `&'static str`.
fn intern_string_limited(s: String) -> &'static str {
    if s.len() <= MAX_STRING_LENGTH {
        return intern_string_owned(s);
    }

    let mut end = MAX_STRING_LENGTH;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    intern_string(&s[..end])
}

fn arbitrary_static_str(u: &mut Unstructured<'_>) -> arbitrary::Result<&'static str> {
    let s: String = u.arbitrary()?;
    Ok(intern_string_limited(s))
}

/// Generate arbitrary struct fields: Vec<(&'static str, Value)>
fn arbitrary_struct_fields(u: &mut Unstructured<'_>) -> arbitrary::Result<Vec<(&'static str, Value)>> {
    let len: usize = u.arbitrary()?;
    let mut fields = Vec::with_capacity(len.min(16));
    for _ in 0..len.min(16) {
        if u.is_empty() {
            break;
        }
        fields.push((arbitrary_static_str(u)?, u.arbitrary()?));
    }
    Ok(fields)
}

impl<'a> Arbitrary<'a> for Value {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let variant = u.int_in_range(0..=28)?;
        Ok(match variant {
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
            17 => Value::Some(Box::new(u.arbitrary()?)),
            18 => Value::Unit,
            19 => Value::UnitStruct {
                name: arbitrary_static_str(u)?,
            },
            20 => Value::NewtypeStruct {
                name: arbitrary_static_str(u)?,
                value: Box::new(u.arbitrary()?),
            },
            21 => Value::NewtypeVariant {
                enum_name: arbitrary_static_str(u)?,
                variant_index: u.arbitrary()?,
                variant: arbitrary_static_str(u)?,
                value: Box::new(u.arbitrary()?),
            },
            22 => Value::Seq(u.arbitrary()?),
            23 => Value::Tuple(u.arbitrary()?),
            24 => Value::TupleStruct {
                name: arbitrary_static_str(u)?,
                fields: u.arbitrary()?,
            },
            25 => Value::TupleVariant {
                enum_name: arbitrary_static_str(u)?,
                variant_index: u.arbitrary()?,
                variant: arbitrary_static_str(u)?,
                fields: u.arbitrary()?,
            },
            26 => Value::Map(u.arbitrary()?),
            27 => Value::Struct {
                name: arbitrary_static_str(u)?,
                fields: arbitrary_struct_fields(u)?,
            },
            28 => Value::StructVariant {
                enum_name: arbitrary_static_str(u)?,
                variant_index: u.arbitrary()?,
                variant: arbitrary_static_str(u)?,
                fields: arbitrary_struct_fields(u)?,
            },
            _ => Value::UnitVariant {
                enum_name: arbitrary_static_str(u)?,
                variant_index: u.arbitrary()?,
                variant: arbitrary_static_str(u)?,
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arbitrary_value_generation() {
        let data = [0u8; 256];
        let mut u = Unstructured::new(&data);
        let value = Value::arbitrary(&mut u);
        assert!(value.is_ok());
    }

    #[test]
    fn test_arbitrary_multiple_values() {
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
        let s1 = intern_string_limited("test_dedup".to_string());
        let s2 = intern_string_limited("test_dedup".to_string());
        assert!(std::ptr::eq(s1, s2));
    }

    #[test]
    fn test_intern_string_truncation() {
        let long_string = "a".repeat(100);
        let interned = intern_string_limited(long_string);
        assert!(interned.len() <= MAX_STRING_LENGTH);
    }

    #[test]
    fn test_intern_string_utf8_boundary() {
        let emojis = "😀".repeat(20);
        let interned = intern_string_limited(emojis);
        assert!(interned.len() <= MAX_STRING_LENGTH);
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
