//! Comprehensive tests for protobuf-wire format implementation.

use protobuf_wire::{
    Message,
    ParseError,
    Record,
    SerializeError,
    Value,
    WireType,
};

// ============================================================================
// Wire Type Tests
// ============================================================================

#[test]
fn test_wire_type_from_u8_valid() {
    assert_eq!(WireType::from_u8(0), Ok(WireType::Varint));
    assert_eq!(WireType::from_u8(1), Ok(WireType::I64));
    assert_eq!(WireType::from_u8(2), Ok(WireType::Len));
    assert_eq!(WireType::from_u8(3), Ok(WireType::SGroup));
    assert_eq!(WireType::from_u8(4), Ok(WireType::EGroup));
    assert_eq!(WireType::from_u8(5), Ok(WireType::I32));
}

#[test]
fn test_wire_type_from_u8_invalid() {
    assert_eq!(WireType::from_u8(6), Err(ParseError::InvalidWireType(6)));
    assert_eq!(WireType::from_u8(7), Err(ParseError::InvalidWireType(7)));
    assert_eq!(
        WireType::from_u8(255),
        Err(ParseError::InvalidWireType(255))
    );
}

// ============================================================================
// Basic Round-Trip Tests
// ============================================================================

#[test]
fn test_empty_message() {
    let msg = Message::new();
    let bytes = msg.serialize().unwrap();
    assert!(bytes.is_empty());
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed.records.len(), 0);
}

#[test]
fn test_single_varint() {
    let msg = Message::from_records(vec![Record::new(1, Value::Varint(42))]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_single_i64() {
    let msg = Message::from_records(vec![Record::new(2, Value::I64(0x1234567890ABCDEF_i64))]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_single_i32() {
    let msg = Message::from_records(vec![Record::new(3, Value::I32(0x12345678_i32))]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_single_len_delimited() {
    let msg = Message::from_records(vec![Record::new(4, Value::LenDelimited(b"hello".to_vec()))]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_empty_len_delimited() {
    let msg = Message::from_records(vec![Record::new(5, Value::LenDelimited(vec![]))]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_simple_group() {
    let msg = Message::from_records(vec![Record::new(
        6,
        Value::Group(vec![Record::new(1, Value::Varint(100))]),
    )]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_empty_group() {
    let msg = Message::from_records(vec![Record::new(7, Value::Group(vec![]))]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

// ============================================================================
// Varint Edge Cases
// ============================================================================

#[test]
fn test_varint_zero() {
    let msg = Message::from_records(vec![Record::new(1, Value::Varint(0))]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_varint_max_single_byte() {
    let msg = Message::from_records(vec![Record::new(1, Value::Varint(127))]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_varint_min_two_bytes() {
    let msg = Message::from_records(vec![Record::new(1, Value::Varint(128))]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_varint_max_value() {
    let msg = Message::from_records(vec![Record::new(1, Value::Varint(u64::MAX))]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

// ============================================================================
// I64/I32 Edge Cases
// ============================================================================

#[test]
fn test_i64_negative() {
    let msg = Message::from_records(vec![Record::new(1, Value::I64(-1))]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_i64_min() {
    let msg = Message::from_records(vec![Record::new(1, Value::I64(i64::MIN))]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_i64_max() {
    let msg = Message::from_records(vec![Record::new(1, Value::I64(i64::MAX))]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_i32_negative() {
    let msg = Message::from_records(vec![Record::new(1, Value::I32(-1))]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_i32_min() {
    let msg = Message::from_records(vec![Record::new(1, Value::I32(i32::MIN))]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_i32_max() {
    let msg = Message::from_records(vec![Record::new(1, Value::I32(i32::MAX))]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

// ============================================================================
// Field Number Edge Cases
// ============================================================================

#[test]
fn test_field_number_one() {
    let msg = Message::from_records(vec![Record::new(1, Value::Varint(42))]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_field_number_max() {
    // Max field number is 2^29 - 1 = 536870911
    let msg = Message::from_records(vec![Record::new(536_870_911, Value::Varint(42))]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_field_number_zero_serialize_error() {
    let msg = Message::from_records(vec![Record::new(0, Value::Varint(42))]);
    let result = msg.serialize();
    assert_eq!(result, Err(SerializeError::InvalidFieldNumber(0)));
}

#[test]
fn test_field_number_overflow_serialize_error() {
    let msg = Message::from_records(vec![Record::new(536_870_912, Value::Varint(42))]);
    let result = msg.serialize();
    assert_eq!(result, Err(SerializeError::InvalidFieldNumber(536_870_912)));
}

// ============================================================================
// Multiple Fields
// ============================================================================

#[test]
fn test_multiple_fields_different_types() {
    let msg = Message::from_records(vec![
        Record::new(1, Value::Varint(42)),
        Record::new(2, Value::I64(0x123456789ABCDEF0_i64)),
        Record::new(3, Value::I32(0x12345678_i32)),
        Record::new(4, Value::LenDelimited(b"test".to_vec())),
    ]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_repeated_field_number() {
    // Same field number can appear multiple times
    let msg = Message::from_records(vec![
        Record::new(1, Value::Varint(1)),
        Record::new(1, Value::Varint(2)),
        Record::new(1, Value::Varint(3)),
    ]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_field_order_preserved() {
    let msg = Message::from_records(vec![
        Record::new(3, Value::Varint(3)),
        Record::new(1, Value::Varint(1)),
        Record::new(2, Value::Varint(2)),
    ]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    // Field order should be preserved exactly
    assert_eq!(parsed.records[0].field_number, 3);
    assert_eq!(parsed.records[1].field_number, 1);
    assert_eq!(parsed.records[2].field_number, 2);
}

// ============================================================================
// Nested Groups
// ============================================================================

#[test]
fn test_nested_groups() {
    let msg = Message::from_records(vec![Record::new(
        1,
        Value::Group(vec![Record::new(
            2,
            Value::Group(vec![Record::new(3, Value::Varint(42))]),
        )]),
    )]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_group_with_multiple_fields() {
    let msg = Message::from_records(vec![Record::new(
        1,
        Value::Group(vec![
            Record::new(10, Value::Varint(100)),
            Record::new(20, Value::LenDelimited(b"nested".to_vec())),
            Record::new(30, Value::I64(12345)),
        ]),
    )]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_multiple_groups_same_level() {
    let msg = Message::from_records(vec![
        Record::new(1, Value::Group(vec![Record::new(10, Value::Varint(1))])),
        Record::new(2, Value::Group(vec![Record::new(20, Value::Varint(2))])),
    ]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

// ============================================================================
// Parse Errors
// ============================================================================

#[test]
fn test_parse_unexpected_eof_in_varint() {
    // Varint with continuation bit but no next byte
    let bytes = vec![0x08, 0x80]; // tag for field 1 varint, then incomplete varint
    let result = Message::parse(&bytes);
    assert_eq!(result, Err(ParseError::UnexpectedEof));
}

#[test]
fn test_parse_unexpected_eof_in_i64() {
    // Tag for I64 but not enough bytes
    let bytes = vec![0x09, 0x01, 0x02, 0x03]; // field 1 I64, only 3 data bytes
    let result = Message::parse(&bytes);
    assert_eq!(result, Err(ParseError::UnexpectedEof));
}

#[test]
fn test_parse_unexpected_eof_in_i32() {
    // Tag for I32 but not enough bytes
    let bytes = vec![0x0D, 0x01, 0x02]; // field 1 I32, only 2 data bytes
    let result = Message::parse(&bytes);
    assert_eq!(result, Err(ParseError::UnexpectedEof));
}

#[test]
fn test_parse_unexpected_eof_in_len_delimited() {
    // Length says 10 bytes but only 5 provided
    let bytes = vec![0x0A, 0x0A, 0x01, 0x02, 0x03, 0x04, 0x05]; // field 1 len, length 10, 5 bytes
    let result = Message::parse(&bytes);
    assert_eq!(result, Err(ParseError::UnexpectedEof));
}

#[test]
fn test_parse_invalid_wire_type() {
    // Wire type 6 is invalid
    let bytes = vec![0x0E]; // field 1, wire type 6
    let result = Message::parse(&bytes);
    assert_eq!(result, Err(ParseError::InvalidWireType(6)));
}

#[test]
fn test_parse_invalid_wire_type_7() {
    // Wire type 7 is invalid
    let bytes = vec![0x0F]; // field 1, wire type 7
    let result = Message::parse(&bytes);
    assert_eq!(result, Err(ParseError::InvalidWireType(7)));
}

#[test]
fn test_parse_field_number_zero() {
    // Field number 0 is invalid
    let bytes = vec![0x00]; // field 0, wire type 0
    let result = Message::parse(&bytes);
    assert_eq!(result, Err(ParseError::InvalidFieldNumber));
}

#[test]
fn test_parse_unterminated_group() {
    // SGROUP without matching EGROUP
    let bytes = vec![0x0B, 0x08, 0x01]; // field 1 SGROUP, field 1 varint 1
    let result = Message::parse(&bytes);
    assert_eq!(
        result,
        Err(ParseError::UnterminatedGroup { field_number: 1 })
    );
}

#[test]
fn test_parse_mismatched_group_end() {
    // SGROUP for field 1, EGROUP for field 2
    let bytes = vec![0x0B, 0x14]; // field 1 SGROUP, field 2 EGROUP
    let result = Message::parse(&bytes);
    assert_eq!(
        result,
        Err(ParseError::MismatchedGroupEnd {
            expected: 1,
            found: 2
        })
    );
}

#[test]
fn test_parse_egroup_without_sgroup() {
    // EGROUP without any SGROUP
    let bytes = vec![0x0C]; // field 1 EGROUP
    let result = Message::parse(&bytes);
    assert_eq!(
        result,
        Err(ParseError::MismatchedGroupEnd {
            expected: 0,
            found: 1
        })
    );
}

// ============================================================================
// Binary Format Verification
// ============================================================================

#[test]
fn test_varint_encoding_format() {
    // Verify exact wire format for varint
    let msg = Message::from_records(vec![Record::new(1, Value::Varint(150))]);
    let bytes = msg.serialize().unwrap();
    // Tag: field 1, wire type 0 = (1 << 3) | 0 = 0x08
    // Value 150 = 10010110 binary, encoded as 0x96 0x01
    assert_eq!(bytes, vec![0x08, 0x96, 0x01]);
}

#[test]
fn test_i64_encoding_format() {
    // Verify exact wire format for I64
    let msg = Message::from_records(vec![Record::new(1, Value::I64(1))]);
    let bytes = msg.serialize().unwrap();
    // Tag: field 1, wire type 1 = (1 << 3) | 1 = 0x09
    // Value: 1 as little-endian 8 bytes
    assert_eq!(bytes, vec![
        0x09, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00
    ]);
}

#[test]
fn test_i32_encoding_format() {
    // Verify exact wire format for I32
    let msg = Message::from_records(vec![Record::new(1, Value::I32(1))]);
    let bytes = msg.serialize().unwrap();
    // Tag: field 1, wire type 5 = (1 << 3) | 5 = 0x0D
    // Value: 1 as little-endian 4 bytes
    assert_eq!(bytes, vec![0x0D, 0x01, 0x00, 0x00, 0x00]);
}

#[test]
fn test_len_delimited_encoding_format() {
    // Verify exact wire format for length-delimited
    let msg = Message::from_records(vec![Record::new(1, Value::LenDelimited(b"ab".to_vec()))]);
    let bytes = msg.serialize().unwrap();
    // Tag: field 1, wire type 2 = (1 << 3) | 2 = 0x0A
    // Length: 2 as varint = 0x02
    // Data: 'a', 'b'
    assert_eq!(bytes, vec![0x0A, 0x02, 0x61, 0x62]);
}

#[test]
fn test_group_encoding_format() {
    // Verify exact wire format for groups
    let msg = Message::from_records(vec![Record::new(
        1,
        Value::Group(vec![Record::new(2, Value::Varint(3))]),
    )]);
    let bytes = msg.serialize().unwrap();
    // SGROUP tag: field 1, wire type 3 = (1 << 3) | 3 = 0x0B
    // Inner record tag: field 2, wire type 0 = (2 << 3) | 0 = 0x10
    // Inner value: 3 = 0x03
    // EGROUP tag: field 1, wire type 4 = (1 << 3) | 4 = 0x0C
    assert_eq!(bytes, vec![0x0B, 0x10, 0x03, 0x0C]);
}

// ============================================================================
// Large Data Tests
// ============================================================================

#[test]
fn test_large_len_delimited() {
    let data = vec![0xAB; 10000];
    let msg = Message::from_records(vec![Record::new(1, Value::LenDelimited(data.clone()))]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_many_fields() {
    let records: Vec<_> = (1..=1000)
        .map(|i| Record::new(i, Value::Varint(i as u64)))
        .collect();
    let msg = Message::from_records(records);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_deeply_nested_groups() {
    // Create 10 levels of nesting
    let mut inner = vec![Record::new(1, Value::Varint(42))];
    for i in 2..=10 {
        inner = vec![Record::new(i, Value::Group(inner))];
    }
    let msg = Message::from_records(inner);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

// ============================================================================
// Non-canonical Varint Parsing
// ============================================================================

#[test]
fn test_parse_non_canonical_varint_accepted() {
    // Non-canonical encoding of 1: 0x81 0x00 instead of 0x01
    // Tag (field 1, varint): 0x08
    // Value: 0x81 0x00 (non-canonical encoding of 1)
    let bytes = vec![0x08, 0x81, 0x00];
    let result = Message::parse(&bytes).unwrap();
    assert_eq!(result.records.len(), 1);
    assert_eq!(result.records[0].field_number, 1);
    assert_eq!(result.records[0].value, Value::Varint(1));
}

#[test]
fn test_roundtrip_normalizes_varints() {
    // Parse non-canonical, serialize produces canonical
    let non_canonical = vec![0x08, 0x81, 0x00]; // field 1, varint 1 (non-canonical)
    let parsed = Message::parse(&non_canonical).unwrap();
    let serialized = parsed.serialize().unwrap();
    // Canonical encoding of field 1, varint 1
    assert_eq!(serialized, vec![0x08, 0x01]);
}

// ============================================================================
// Complex Realistic Examples
// ============================================================================

#[test]
fn test_realistic_message() {
    // Simulate a message like: { id: 1, name: "test", nested: { value: 42 } }
    let msg = Message::from_records(vec![
        Record::new(1, Value::Varint(1)),                      // id
        Record::new(2, Value::LenDelimited(b"test".to_vec())), // name
        Record::new(
            3,
            Value::Group(vec![Record::new(1, Value::Varint(42))]), // nested.value
        ),
    ]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

#[test]
fn test_mixed_wire_types_in_group() {
    let msg = Message::from_records(vec![Record::new(
        1,
        Value::Group(vec![
            Record::new(1, Value::Varint(0)),
            Record::new(2, Value::I64(-1)),
            Record::new(3, Value::I32(-1)),
            Record::new(4, Value::LenDelimited(vec![0, 1, 2, 3])),
            Record::new(5, Value::Group(vec![])),
        ]),
    )]);
    let bytes = msg.serialize().unwrap();
    let parsed = Message::parse(&bytes).unwrap();
    assert_eq!(parsed, msg);
}

// ============================================================================
// Error Display Tests
// ============================================================================

#[test]
fn test_parse_error_display() {
    assert_eq!(
        format!("{}", ParseError::UnexpectedEof),
        "unexpected end of input"
    );
    assert_eq!(
        format!("{}", ParseError::InvalidVarint),
        "invalid varint encoding"
    );
    assert_eq!(
        format!("{}", ParseError::InvalidWireType(6)),
        "invalid wire type: 6"
    );
    assert_eq!(
        format!("{}", ParseError::InvalidFieldNumber),
        "field number must be in range 1 to 536870911"
    );
    assert_eq!(
        format!("{}", ParseError::MismatchedGroupEnd {
            expected: 1,
            found: 2
        }),
        "mismatched group end: expected field 1, found field 2"
    );
    assert_eq!(
        format!("{}", ParseError::UnterminatedGroup { field_number: 5 }),
        "unterminated group at field 5"
    );
    assert_eq!(
        format!("{}", ParseError::LengthOverflow),
        "length prefix exceeds available data"
    );
}

#[test]
fn test_serialize_error_display() {
    assert_eq!(
        format!("{}", SerializeError::InvalidFieldNumber(0)),
        "invalid field number 0: must be in range 1 to 536870911"
    );
}
