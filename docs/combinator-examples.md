# Nom Combinator Examples for JEB85

This document shows practical examples of how the JEB85 combinators work and can be composed with other nom parsers.

## Quick Reference: The Three Combinators

```rust
// 1. parse_jeb85 - Routes to text or binary mode based on prefix
fn parse_jeb85(input: &[u8]) -> IResult<&[u8], Vec<u8>>

// 2. parse_text_mode - Validates UTF-8 and control characters
fn parse_text_mode(input: &[u8]) -> IResult<&[u8], Vec<u8>>

// 3. parse_binary_mode - Decodes Z85 and raw chunks
fn parse_binary_mode(input: &[u8]) -> IResult<&[u8], Vec<u8>>
```

## Example 1: Basic Usage (Public API)

The simplest way to use JEB85 is through the public `decode()` function:

```rust
use json_encoded_binary::{encode, decode};

// Text mode - simple UTF-8 string
let text = "Hello, World!";
let decoded = decode(text).unwrap();
assert_eq!(decoded, text.as_bytes());

// Binary mode - has \b prefix
let binary_data = vec![0x00, 0x01, 0x02, 0xFF];
let encoded = encode(&binary_data);  // Returns "\b..." with Z85 encoding
let decoded = decode(&encoded).unwrap();
assert_eq!(decoded, binary_data);
```

## Example 2: Understanding the Combinator Flow

Here's how the internal combinators work together:

```rust
// Input: "Hello"
parse_jeb85(b"Hello")
  → Checks first byte: not 0x08
  → Routes to parse_text_mode(b"Hello")
     → Validates UTF-8: ✓
     → Checks size (≤64KB): ✓
     → Checks control chars: ✓
     → Returns: Ok((&b""[..], vec![72, 101, 108, 108, 111]))

// Input: "\bHello" (with backspace prefix)
parse_jeb85(b"\x08Hello")
  → Checks first byte: 0x08 found!
  → Strips prefix with tag(&[0x08])
  → Routes to parse_binary_mode(b"Hello")
     → Parses chunks (in this case, raw ASCII)
     → Returns: Ok((&b""[..], vec![...]))
```

## Example 3: Composing with `alt` (Alternative Parsers)

Try multiple encoding schemes:

```rust
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::IResult;

fn parse_any_format(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    alt((
        parse_base64_tagged,  // "base64:..."
        parse_hex_tagged,     // "hex:..."
        parse_jeb85_tagged,   // "jeb:..."
    ))(input)
}

fn parse_jeb85_tagged(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (input, _) = tag(b"jeb:")(input)?;

    // Convert remaining bytes to string for JEB85 decoding
    let data_str = std::str::from_utf8(input)
        .map_err(|_| nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Char
        )))?;

    let decoded = json_encoded_binary::decode(data_str)
        .map_err(|_| nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify
        )))?;

    Ok((&b""[..], decoded))
}

// Usage:
// parse_any_format(b"jeb:Hello") → Ok(vec![72, 101, 108, 108, 111])
```

## Example 4: Composing with `tuple` (Sequential Parsing)

Parse a protocol message with header and JEB85 payload:

```rust
use nom::sequence::tuple;
use nom::bytes::complete::{tag, take};
use nom::number::complete::be_u16;
use nom::IResult;

// Protocol: [MAGIC:2]["JEB"][LENGTH:2][DATA:LENGTH]
fn parse_message(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    // Parse header
    let (input, (_, _, length)) = tuple((
        tag(b"MG"),        // Magic bytes
        tag(b"JEB"),       // Format identifier
        be_u16,            // Length as big-endian u16
    ))(input)?;

    // Take exactly 'length' bytes
    let (input, payload_bytes) = take(length as usize)(input)?;

    // Decode as JEB85
    let payload_str = std::str::from_utf8(payload_bytes)
        .map_err(|_| nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Char
        )))?;

    let decoded = json_encoded_binary::decode(payload_str)
        .map_err(|_| nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify
        )))?;

    Ok((input, decoded))
}

// Example:
// Input: b"MGJEB\x00\x05Hello"
//         ^^^^^ ^^^ ^^^^^ ^^^^^
//         magic fmt len   data
// Output: Ok((&b""[..], vec![72, 101, 108, 108, 111]))
```

## Example 5: Composing with `many0` (Repeated Parsing)

Parse multiple JEB85 values separated by newlines:

```rust
use nom::multi::many0;
use nom::bytes::complete::{tag, take_until};
use nom::IResult;

fn parse_jeb85_lines(input: &[u8]) -> IResult<&[u8], Vec<Vec<u8>>> {
    many0(|input| {
        // Parse up to newline
        let (input, line_bytes) = take_until("\n")(input)?;
        let (input, _) = tag(b"\n")(input)?;

        // Decode the line as JEB85
        let line_str = std::str::from_utf8(line_bytes)
            .map_err(|_| nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Char
            )))?;

        let decoded = json_encoded_binary::decode(line_str)
            .map_err(|_| nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Verify
            )))?;

        Ok((input, decoded))
    })(input)
}

// Example:
// Input: b"Hello\nWorld\nTest\n"
// Output: Ok((
//     &b""[..],
//     vec![
//         vec![72, 101, 108, 108, 111],  // "Hello"
//         vec![87, 111, 114, 108, 100],  // "World"
//         vec![84, 101, 115, 116],       // "Test"
//     ]
// ))
```

## Example 6: Composing with `opt` (Optional Parsing)

Parse optional JEB85 encoding:

```rust
use nom::combinator::opt;
use nom::bytes::complete::{tag, take_until};
use nom::sequence::preceded;
use nom::IResult;

// Parse: "key:value" or "key:!jeb85!value"
fn parse_key_value(input: &[u8]) -> IResult<&[u8], (&[u8], Vec<u8>)> {
    // Parse key
    let (input, key) = take_until(":")(input)?;
    let (input, _) = tag(b":")(input)?;

    // Check for JEB85 marker
    let (input, is_jeb85) = opt(tag(b"!jeb85!"))(input)?;

    // Get the value
    let (input, value_bytes) = nom::combinator::rest(input)?;

    let value = if is_jeb85.is_some() {
        // Decode as JEB85
        let value_str = std::str::from_utf8(value_bytes)
            .map_err(|_| nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Char
            )))?;

        json_encoded_binary::decode(value_str)
            .map_err(|_| nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Verify
            )))?
    } else {
        // Use as-is
        value_bytes.to_vec()
    };

    Ok((&b""[..], (key, value)))
}

// Examples:
// parse_key_value(b"name:alice")           → ("name", b"alice")
// parse_key_value(b"data:!jeb85!Hello")    → ("data", b"Hello")
```

## Example 7: Integration with serde_json

Parse JSON and decode embedded JEB85 values:

```rust
use serde_json::Value;

fn decode_jeb85_in_json(json_str: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let mut json: Value = serde_json::from_str(json_str)?;

    // Process all string values that might be JEB85-encoded
    if let Some(obj) = json.as_object_mut() {
        for (_key, value) in obj.iter_mut() {
            if let Some(s) = value.as_str() {
                // Try to decode as JEB85
                if let Ok(decoded) = json_encoded_binary::decode(s) {
                    // If it's valid UTF-8, replace with decoded string
                    if let Ok(decoded_str) = String::from_utf8(decoded.clone()) {
                        *value = Value::String(decoded_str);
                    } else {
                        // Otherwise, store as base64 or hex
                        *value = Value::String(format!("binary:{:?}", decoded));
                    }
                }
            }
        }
    }

    Ok(json)
}

// Example:
// Input:  {"name": "test", "data": "\bHello"}
// Output: {"name": "test", "data": "Hello"} (decoded)
```

## Example 8: Custom Validation Layer

Wrap JEB85 parsing with additional validation:

```rust
use nom::IResult;

fn parse_validated_jeb85_text(input: &str) -> IResult<&str, String> {
    // Decode
    let decoded = json_encoded_binary::decode(input)
        .map_err(|_| nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify
        )))?;

    // Ensure it's valid UTF-8
    let text = String::from_utf8(decoded)
        .map_err(|_| nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Char
        )))?;

    // Custom validation: must not be empty and must not contain null bytes
    if text.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify
        )));
    }

    if text.contains('\0') {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Char
        )));
    }

    Ok(("", text))
}
```

## How the Split Architecture Helps

The split combinator design enables:

### 1. **Independent Testing**
```rust
#[test]
fn test_text_mode_only() {
    // Test UTF-8 validation without binary logic
    assert!(parse_text_mode(b"Hello").is_ok());
    assert!(parse_text_mode(b"\x00").is_err());
}

#[test]
fn test_binary_mode_only() {
    // Test Z85 decoding without text validation
    let encoded = encode_z85_block(&[0x86, 0x4F, 0xD2, 0x6F]);
    assert_eq!(parse_z85_block(&encoded).unwrap().1, vec![0x86, 0x4F, 0xD2, 0x6F]);
}
```

### 2. **Clear Error Messages**
```rust
// Text mode errors are specific to UTF-8/control chars
parse_text_mode(b"\xFF\xFE") → InvalidUtf8

// Binary mode errors are specific to encoding
parse_binary_mode(b"!!!") → InvalidZ85Character
```

### 3. **Easy Composition**
```rust
// You can use just the text validator in other contexts
fn validate_utf8_field(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    parse_text_mode(input)  // Reuse the validator!
}
```

## Summary

The split combinator architecture provides:

- ✅ **Modularity**: Each combinator does one thing well
- ✅ **Composability**: Easy to combine with other nom parsers
- ✅ **Testability**: Independent unit tests for each component
- ✅ **Clarity**: Clean separation between text and binary paths
- ✅ **Error handling**: Specific errors for each validation stage

The key insight: **Small, focused combinators are easier to understand, test, and compose than monolithic parsers.**
