//! Parsing logic for the protobuf wire format.

use crate::{
    error::ParseError,
    types::{
        Message,
        Record,
        Value,
    },
    varint::decode_varint,
    wire_type::WireType,
};

/// Maximum valid field number (2^29 - 1).
const MAX_FIELD_NUMBER: u32 = 536_870_911;

/// Maximum nesting depth for groups to prevent stack overflow.
const MAX_DEPTH: usize = 1024;

impl Message {
    /// Parse a wire-format message from bytes.
    pub fn parse(bytes: &[u8]) -> Result<Self, ParseError> {
        let mut parser = Parser::new(bytes);
        parser.parse_message(None, 0)
    }
}

/// Internal parser state.
struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Parser { bytes, pos: 0 }
    }

    /// Returns true if we've reached the end of input.
    fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }

    /// Returns remaining bytes.
    fn remaining(&self) -> &[u8] {
        &self.bytes[self.pos..]
    }

    /// Read a varint from the current position.
    fn read_varint(&mut self) -> Result<u64, ParseError> {
        let (value, len) = decode_varint(self.remaining())?;
        self.pos += len;
        Ok(value)
    }

    /// Read exactly n bytes.
    fn read_bytes(&mut self, n: usize) -> Result<&'a [u8], ParseError> {
        let end = self.pos.checked_add(n).ok_or(ParseError::LengthOverflow)?;
        if end > self.bytes.len() {
            return Err(ParseError::UnexpectedEof);
        }
        let result = &self.bytes[self.pos..end];
        self.pos = end;
        Ok(result)
    }

    /// Parse a tag and return (field_number, wire_type).
    fn read_tag(&mut self) -> Result<(u32, WireType), ParseError> {
        let tag = self.read_varint()?;

        let wire_type_bits = (tag & 0x07) as u8;
        let wire_type = WireType::from_u8(wire_type_bits)?;

        let field_number = (tag >> 3) as u32;
        if field_number == 0 || field_number > MAX_FIELD_NUMBER {
            return Err(ParseError::InvalidFieldNumber);
        }

        Ok((field_number, wire_type))
    }

    /// Parse a message, optionally stopping at an EGROUP with the given field
    /// number.
    fn parse_message(
        &mut self,
        group_field: Option<u32>,
        depth: usize,
    ) -> Result<Message, ParseError> {
        if depth > MAX_DEPTH {
            return Err(ParseError::NestingTooDeep);
        }

        let mut records = Vec::new();

        while !self.is_eof() {
            let (field_number, wire_type) = self.read_tag()?;

            // Check for group end
            if wire_type == WireType::EGroup {
                match group_field {
                    Some(expected) if expected == field_number => {
                        // Matching EGROUP found, return the message
                        return Ok(Message { records });
                    }
                    Some(expected) => {
                        return Err(ParseError::MismatchedGroupEnd {
                            expected,
                            found: field_number,
                        });
                    }
                    None => {
                        // EGROUP outside of a group is invalid
                        return Err(ParseError::MismatchedGroupEnd {
                            expected: 0,
                            found: field_number,
                        });
                    }
                }
            }

            let value = self.read_value(field_number, wire_type, depth)?;
            records.push(Record {
                field_number,
                value,
            });
        }

        // Check if we were expecting a group end
        if let Some(field_number) = group_field {
            return Err(ParseError::UnterminatedGroup { field_number });
        }

        Ok(Message { records })
    }

    /// Read a value based on wire type.
    fn read_value(
        &mut self,
        field_number: u32,
        wire_type: WireType,
        depth: usize,
    ) -> Result<Value, ParseError> {
        match wire_type {
            WireType::Varint => {
                let value = self.read_varint()?;
                Ok(Value::Varint(value))
            }
            WireType::I64 => {
                let bytes = self.read_bytes(8)?;
                let value = i64::from_le_bytes(bytes.try_into().unwrap());
                Ok(Value::I64(value))
            }
            WireType::I32 => {
                let bytes = self.read_bytes(4)?;
                let value = i32::from_le_bytes(bytes.try_into().unwrap());
                Ok(Value::I32(value))
            }
            WireType::Len => {
                let len = self.read_varint()?;
                // Check for overflow
                let len = usize::try_from(len).map_err(|_| ParseError::LengthOverflow)?;
                let bytes = self.read_bytes(len)?;
                Ok(Value::LenDelimited(bytes.to_vec()))
            }
            WireType::SGroup => {
                // Recursively parse until matching EGROUP
                let inner = self.parse_message(Some(field_number), depth + 1)?;
                Ok(Value::Group(inner.records))
            }
            WireType::EGroup => {
                // This is handled in parse_message, shouldn't reach here
                unreachable!("EGroup handled in parse_message")
            }
        }
    }
}
