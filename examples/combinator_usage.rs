//! Examples of using the JEB85 combinators
//!
//! This demonstrates how nom combinators work and can be composed.

use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    combinator::{map, opt},
    multi::many0,
    sequence::{delimited, preceded, tuple},
    IResult,
};

// Re-export the parsing functions from json-encoded-binary for demonstration
// (In real code, these would be pub and imported from the library)

/// Example 1: Basic combinator usage
///
/// The simplest use case - parse a single JEB85 value
fn example_basic_parsing() {
    println!("=== Example 1: Basic Parsing ===\n");

    // Text mode example
    let text_input = "Hello, World!";
    match json_encoded_binary::decode(text_input) {
        Ok(bytes) => {
            println!("Text mode input: {:?}", text_input);
            println!("Decoded bytes: {:?}", bytes);
            println!("As UTF-8: {:?}\n", String::from_utf8(bytes).unwrap());
        }
        Err(e) => println!("Error: {}\n", e),
    }

    // Binary mode example (with backspace prefix)
    let binary_input = "\x08Hello"; // \b prefix makes it binary mode
    match json_encoded_binary::decode(binary_input) {
        Ok(bytes) => {
            println!("Binary mode input: {:?}", binary_input);
            println!("Decoded bytes: {:?}\n", bytes);
        }
        Err(e) => println!("Error: {}\n", e),
    }
}

/// Example 2: Composing with other nom combinators
///
/// Parse a JSON string that contains a JEB85-encoded value
fn parse_json_with_jeb85(input: &str) -> IResult<&str, String> {
    // Parse: {"data": "<JEB85_VALUE>"}
    let (input, _) = tag("{\"data\": \"")(input)?;
    let (input, jeb_str) = take_until("\"")(input)?;
    let (input, _) = tag("\"}")(input)?;

    // Decode the JEB85 value
    let decoded = json_encoded_binary::decode(jeb_str)
        .map_err(|_| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify)))?;

    Ok((input, String::from_utf8_lossy(&decoded).to_string()))
}

fn example_composition() {
    println!("=== Example 2: Composition with JSON ===\n");

    let input = r#"{"data": "Hello, World!"}"#;
    match parse_json_with_jeb85(input) {
        Ok((remaining, data)) => {
            println!("Input: {}", input);
            println!("Parsed data: {}", data);
            println!("Remaining: {:?}\n", remaining);
        }
        Err(e) => println!("Error: {:?}\n", e),
    }
}

/// Example 3: Parsing multiple JEB85 values
///
/// Parse a stream of JEB85 values separated by newlines
fn parse_jeb85_stream(input: &str) -> IResult<&str, Vec<Vec<u8>>> {
    many0(|input| {
        // Parse a line
        let (input, line) = take_until("\n")(input)?;
        let (input, _) = tag("\n")(input)?;

        // Decode the JEB85 value
        let decoded = json_encoded_binary::decode(line)
            .map_err(|_| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify)))?;

        Ok((input, decoded))
    })(input)
}

fn example_stream_parsing() {
    println!("=== Example 3: Stream Parsing ===\n");

    let input = "Hello\nWorld\nFrom Rust\n";
    match parse_jeb85_stream(input) {
        Ok((remaining, values)) => {
            println!("Input: {:?}", input);
            println!("Parsed {} values:", values.len());
            for (i, val) in values.iter().enumerate() {
                println!("  [{}]: {:?}", i, String::from_utf8_lossy(val));
            }
            println!("Remaining: {:?}\n", remaining);
        }
        Err(e) => println!("Error: {:?}\n", e),
    }
}

/// Example 4: Protocol parser with headers
///
/// Parse a custom protocol: "JEB85:<length>:<data>"
fn parse_jeb85_protocol(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    // Parse header
    let (input, _) = tag(b"JEB85:")(input)?;

    // Parse length (as ASCII digits)
    let (input, length_str) = take_until(&b":"[..])(input)?;
    let (input, _) = tag(b":")(input)?;

    let length = std::str::from_utf8(length_str)
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .ok_or_else(|| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Digit)))?;

    // Take exactly 'length' bytes
    let (input, data_bytes) = nom::bytes::complete::take(length)(input)?;

    // Decode as JEB85
    let data_str = std::str::from_utf8(data_bytes)
        .map_err(|_| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Char)))?;

    let decoded = json_encoded_binary::decode(data_str)
        .map_err(|_| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify)))?;

    Ok((input, decoded))
}

fn example_protocol() {
    println!("=== Example 4: Custom Protocol ===\n");

    let input = b"JEB85:5:Hello";
    match parse_jeb85_protocol(input) {
        Ok((remaining, data)) => {
            println!("Input: {:?}", std::str::from_utf8(input).unwrap());
            println!("Decoded: {:?}", String::from_utf8_lossy(&data));
            println!("Remaining: {:?}\n", remaining);
        }
        Err(e) => println!("Error: {:?}\n", e),
    }
}

/// Example 5: Optional JEB85 field
///
/// Parse a structure where JEB85 encoding is optional
fn parse_optional_jeb85(input: &str) -> IResult<&str, (String, Option<Vec<u8>>)> {
    // Parse: "name:value" or "name:!jeb85!value"
    let (input, name) = take_until(":")(input)?;
    let (input, _) = tag(":")(input)?;

    // Check for JEB85 marker
    let (input, is_jeb85) = opt(tag("!jeb85!"))(input)?;

    let (input, value) = if is_jeb85.is_some() {
        // Parse as JEB85
        let (input, jeb_str) = nom::combinator::rest(input)?;
        let decoded = json_encoded_binary::decode(jeb_str)
            .map_err(|_| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify)))?;
        (input, Some(decoded))
    } else {
        // Plain text
        (input, None)
    };

    Ok((input, (name.to_string(), value)))
}

fn example_optional() {
    println!("=== Example 5: Optional Encoding ===\n");

    let plain = "username:alice";
    let encoded = "data:!jeb85!Hello";

    match parse_optional_jeb85(plain) {
        Ok((_, (name, value))) => {
            println!("Plain - Name: {}, Value: {:?}", name, value);
        }
        Err(e) => println!("Error: {:?}", e),
    }

    match parse_optional_jeb85(encoded) {
        Ok((_, (name, value))) => {
            println!("Encoded - Name: {}, Value: {:?}\n", name,
                value.map(|v| String::from_utf8_lossy(&v).to_string()));
        }
        Err(e) => println!("Error: {:?}", e),
    }
}

/// Example 6: Integration with serde_json
///
/// Parse JSON and decode JEB85 values within it
fn example_serde_integration() {
    println!("=== Example 6: Serde JSON Integration ===\n");

    use serde_json::Value;

    let json_str = r#"{
        "name": "test",
        "data": "Hello, World!",
        "binary": "\bHello"
    }"#;

    let json: Value = serde_json::from_str(json_str).unwrap();

    println!("Original JSON: {}", json);

    // Process JEB85-encoded fields
    if let Some(data) = json.get("data").and_then(|v| v.as_str()) {
        if let Ok(decoded) = json_encoded_binary::decode(data) {
            println!("Decoded 'data': {:?}", String::from_utf8_lossy(&decoded));
        }
    }

    if let Some(binary) = json.get("binary").and_then(|v| v.as_str()) {
        if let Ok(decoded) = json_encoded_binary::decode(binary) {
            println!("Decoded 'binary': {:?}\n", decoded);
        }
    }
}

/// Example 7: Building a higher-level combinator
///
/// Create a combinator that validates and transforms JEB85 data
fn parse_validated_text(input: &str) -> IResult<&str, String> {
    // Decode JEB85
    let decoded = json_encoded_binary::decode(input)
        .map_err(|_| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify)))?;

    // Ensure it's valid UTF-8
    let text = String::from_utf8(decoded)
        .map_err(|_| nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Char)))?;

    // Additional validation: must not be empty
    if text.is_empty() {
        return Err(nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify)));
    }

    Ok(("", text))
}

fn example_validation() {
    println!("=== Example 7: Validated Parsing ===\n");

    let valid = "Hello";
    let empty = "";

    match parse_validated_text(valid) {
        Ok((_, text)) => println!("Valid: {:?}", text),
        Err(e) => println!("Error: {:?}", e),
    }

    match parse_validated_text(empty) {
        Ok((_, text)) => println!("Empty: {:?}", text),
        Err(e) => println!("Empty failed validation (expected)\n"),
    }
}

fn main() {
    example_basic_parsing();
    example_composition();
    example_stream_parsing();
    example_protocol();
    example_optional();
    example_serde_integration();
    example_validation();

    println!("=== Key Takeaways ===\n");
    println!("1. Combinators are composable building blocks");
    println!("2. They can be mixed with other nom parsers");
    println!("3. Error handling flows through the combinator chain");
    println!("4. You can build domain-specific parsers on top");
    println!("5. Integration with other libraries (like serde) is straightforward");
}
