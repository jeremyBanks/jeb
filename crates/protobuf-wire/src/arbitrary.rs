//! Implementation of the `Arbitrary` trait for fuzz testing.

use {
    crate::{
        Message,
        Record,
        Value,
        WireType,
    },
    arbitrary::{
        Arbitrary,
        Unstructured,
    },
};

/// Maximum valid field number (2^29 - 1).
const MAX_FIELD_NUMBER: u32 = 536_870_911;

/// Generate an arbitrary valid field number (1 to MAX_FIELD_NUMBER).
fn arbitrary_field_number(u: &mut Unstructured<'_>) -> arbitrary::Result<u32> {
    let n: u32 = u.arbitrary()?;
    Ok((n % MAX_FIELD_NUMBER) + 1)
}

impl<'a> Arbitrary<'a> for WireType {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let variant = u.int_in_range(0..=5)?;
        Ok(match variant {
            0 => WireType::Varint,
            1 => WireType::I64,
            2 => WireType::Len,
            3 => WireType::SGroup,
            4 => WireType::EGroup,
            5 => WireType::I32,
            _ => unreachable!(),
        })
    }
}

impl<'a> Arbitrary<'a> for Value {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let variant = u.int_in_range(0..=4)?;
        Ok(match variant {
            0 => Value::Varint(u.arbitrary()?),
            1 => Value::I64(u.arbitrary()?),
            2 => Value::I32(u.arbitrary()?),
            3 => Value::LenDelimited(u.arbitrary()?),
            4 => Value::Group(u.arbitrary()?),
            _ => unreachable!(),
        })
    }
}

impl<'a> Arbitrary<'a> for Record {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Record {
            field_number: arbitrary_field_number(u)?,
            value: u.arbitrary()?,
        })
    }
}

impl<'a> Arbitrary<'a> for Message {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Message {
            records: u.arbitrary()?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arbitrary_wire_type() {
        let data = [0u8, 1, 2, 3, 4, 5];
        let mut u = Unstructured::new(&data);
        let wt = WireType::arbitrary(&mut u);
        assert!(wt.is_ok());
    }

    #[test]
    fn test_arbitrary_value() {
        let data = [0u8; 256];
        let mut u = Unstructured::new(&data);
        let value = Value::arbitrary(&mut u);
        assert!(value.is_ok());
    }

    #[test]
    fn test_arbitrary_record_valid_field_number() {
        let data: Vec<u8> = (0..=255).collect();
        let mut u = Unstructured::new(&data);

        for _ in 0..10 {
            if let Ok(record) = Record::arbitrary(&mut u) {
                assert!(record.field_number >= 1);
                assert!(record.field_number <= MAX_FIELD_NUMBER);
            }
        }
    }

    #[test]
    fn test_arbitrary_message() {
        let data = [0u8; 512];
        let mut u = Unstructured::new(&data);
        let msg = Message::arbitrary(&mut u);
        assert!(msg.is_ok());
    }

    #[test]
    fn test_arbitrary_message_roundtrip() {
        let data: Vec<u8> = (0..=255).cycle().take(1024).collect();
        let mut u = Unstructured::new(&data);

        let mut successful_roundtrips = 0;
        for _ in 0..10 {
            if let Ok(msg) = Message::arbitrary(&mut u) {
                if let Ok(bytes) = msg.serialize() {
                    if let Ok(parsed) = Message::parse(&bytes) {
                        assert_eq!(msg, parsed);
                        successful_roundtrips += 1;
                    }
                }
            }
        }
        assert!(successful_roundtrips > 0);
    }

    #[test]
    fn test_field_number_never_zero() {
        let data: Vec<u8> = (0..=255).collect();
        let mut u = Unstructured::new(&data);

        for _ in 0..100 {
            if let Ok(n) = arbitrary_field_number(&mut u) {
                assert!(n >= 1);
                assert!(n <= MAX_FIELD_NUMBER);
            }
        }
    }
}
