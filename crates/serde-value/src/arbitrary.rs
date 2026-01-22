//! Implementation of the `Arbitrary` trait for fuzz testing.

use crate::Value;
use arbitrary::{Arbitrary, Unstructured};

/// A fixed set of static strings for arbitrary generation.
///
/// Since `Value` contains `&'static str` fields, we need to select from
/// a predefined set of strings rather than generating arbitrary ones.
const STATIC_NAMES: &[&str] = &[
    "a", "b", "c", "foo", "bar", "baz", "name", "value", "key", "data", "id", "type", "item",
    "field", "variant", "struct", "enum", "tuple", "option", "result",
];

/// Choose a static string from the predefined set.
fn arbitrary_static_str(u: &mut Unstructured<'_>) -> arbitrary::Result<&'static str> {
    let idx = u.choose_index(STATIC_NAMES.len())?;
    Ok(STATIC_NAMES[idx])
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
    fn test_static_str_selection() {
        let data = [0u8, 1, 2, 3, 4, 5, 6, 7, 8, 9];
        let mut u = Unstructured::new(&data);
        let s = arbitrary_static_str(&mut u);
        assert!(s.is_ok());
        assert!(STATIC_NAMES.contains(&s.unwrap()));
    }
}
