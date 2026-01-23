//! Serialization logic for the protobuf wire format.

use crate::{
    error::SerializeError,
    types::{
        Message,
        Record,
        Value,
    },
    varint::encode_varint_into,
    wire_type::WireType,
};

/// Maximum valid field number (2^29 - 1).
const MAX_FIELD_NUMBER: u32 = 536_870_911;

impl Message {
    /// Serialize to wire format.
    pub fn serialize(&self) -> Result<Vec<u8>, SerializeError> {
        let mut buf = Vec::new();
        serialize_records(&self.records, &mut buf)?;
        Ok(buf)
    }
}

/// Validate a field number.
fn validate_field_number(field_number: u32) -> Result<(), SerializeError> {
    if field_number == 0 || field_number > MAX_FIELD_NUMBER {
        return Err(SerializeError::InvalidFieldNumber(field_number));
    }
    Ok(())
}

/// Encode a tag (field_number << 3 | wire_type) as a varint.
fn encode_tag(field_number: u32, wire_type: WireType, buf: &mut Vec<u8>) {
    let tag = ((field_number as u64) << 3) | (wire_type as u64);
    encode_varint_into(tag, buf);
}

/// Serialize a slice of records.
fn serialize_records(records: &[Record], buf: &mut Vec<u8>) -> Result<(), SerializeError> {
    for record in records {
        serialize_record(record, buf)?;
    }
    Ok(())
}

/// Serialize a single record.
fn serialize_record(record: &Record, buf: &mut Vec<u8>) -> Result<(), SerializeError> {
    validate_field_number(record.field_number)?;

    match &record.value {
        Value::Varint(v) => {
            encode_tag(record.field_number, WireType::Varint, buf);
            encode_varint_into(*v, buf);
        }
        Value::I64(v) => {
            encode_tag(record.field_number, WireType::I64, buf);
            buf.extend_from_slice(&v.to_le_bytes());
        }
        Value::I32(v) => {
            encode_tag(record.field_number, WireType::I32, buf);
            buf.extend_from_slice(&v.to_le_bytes());
        }
        Value::LenDelimited(bytes) => {
            encode_tag(record.field_number, WireType::Len, buf);
            encode_varint_into(bytes.len() as u64, buf);
            buf.extend_from_slice(bytes);
        }
        Value::Group(records) => {
            // SGROUP tag
            encode_tag(record.field_number, WireType::SGroup, buf);
            // Group contents
            serialize_records(records, buf)?;
            // EGROUP tag
            encode_tag(record.field_number, WireType::EGroup, buf);
        }
    }

    Ok(())
}
