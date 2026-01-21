# Nom Combinators in JEB85: A Detailed Guide

This guide explains how the nom combinators in `json-encoded-binary` work and
how they can be used and composed.

## Table of Contents

1. [Understanding Nom Combinators](#understanding-nom-combinators)
2. [The JEB85 Combinator Architecture](#the-jeb85-combinator-architecture)
3. [Internal Combinators Explained](#internal-combinators-explained)
4. [Composition Patterns](#composition-patterns)
5. [Advanced Usage](#advanced-usage)

---

## Understanding Nom Combinators

### What is a Combinator?

A **combinator** in nom is a function that:

- Takes some input (usually `&[u8]` or `&str`)
- Tries to parse it
- Returns an `IResult<Input, Output>`

```rust
type IResult<I, O> = Result<(I, O), nom::Err<E>>;
//                         ^^^^^^^^  parsing succeeded
//                          |    |
//                          |    +-- parsed value
//                          +------- remaining unparsed input
```

### Simple Example

```rust
use nom::bytes::complete::tag;

// The 'tag' combinator matches an exact byte sequence
let result = tag::<_, _, nom::error::Error<&[u8]>>(b"hello")(b"hello world");
//           ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^  combinator definition
//                                                          ^^^^^^^^^^^^^^^^ input

assert_eq!(result, Ok((&b" world"[..], &b"hello"[..])));
//                      ^^^^^^^^^^^^   ^^^^^^^^^^^
//                      remaining      parsed value
```

### Why Combinators?

Combinators are **composable** - you can combine simple parsers into complex
ones:

```rust
use nom::{
    bytes::complete::tag,
    sequence::tuple,
    IResult,
};

fn parse_greeting(input: &[u8]) -> IResult<&[u8], (&[u8], &[u8])> {
    tuple((
        tag(b"Hello, "),  // Parse "Hello, "
        tag(b"World"),    // Then parse "World"
    ))(input)
}

let result = parse_greeting(b"Hello, World!");
assert_eq!(result, Ok((&b"!"[..], (&b"Hello, "[..], &b"World"[..]))));
```

---

## The JEB85 Combinator Architecture

The JEB85 decoder uses a **three-layer architecture**:

```
┌─────────────────────────────────────┐
│     parse_jeb85 (Router)            │  ← Top level: routes based on prefix
│  - Checks first byte                │
│  - Routes to text or binary mode    │
└─────────┬───────────────────────────┘
          │
          ├─────────────────┬──────────────────┐
          ▼                 ▼                  ▼
┌──────────────────┐ ┌──────────────────┐ ┌──────────────────┐
│ parse_text_mode  │ │parse_binary_mode │ │ tag(&[0x08])     │
│ - Validates UTF-8│ │ - Parses chunks  │ │ - Consumes \b    │
│ - Size check     │ │ - Z85 + raw      │ │                  │
│ - Control chars  │ │                  │ │                  │
└──────────────────┘ └─────────┬────────┘ └──────────────────┘
                               │
                     ┌─────────┴─────────┬────────────────┐
                     ▼                   ▼                ▼
              ┌─────────────┐    ┌──────────────┐  ┌────────────┐
              │parse_z85_   │    │parse_single_ │  │parse_multi_│
              │  block      │    │  raw_block   │  │ raw_block  │
              └─────────────┘    └──────────────┘  └────────────┘
```

---

## Internal Combinators Explained

### 1. `parse_jeb85` - The Router

**Location:** `src/lib.rs:372-383`

```rust
fn parse_jeb85(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    if input.starts_with(&[0x08]) {
        // Binary mode: skip prefix and parse
        let (input, _) = tag(&[0x08])(input)?;
        parse_binary_mode(input)
    } else {
        // Text mode: validate
        parse_text_mode(input)
    }
}
```

**How it works:**

1. Checks if input starts with `\b` (0x08)
2. If yes → strip prefix, parse as binary
3. If no → validate as text

**Usage pattern:**

```rust
let text_result = parse_jeb85(b"Hello");
// Ok((&b""[..], vec![72, 101, 108, 108, 111]))

let binary_result = parse_jeb85(b"\x08Hello");  // \b prefix
// Ok((&b""[..], vec![...encoded data...]))
```

---

### 2. `parse_text_mode` - UTF-8 Validator

**Location:** `src/lib.rs:334-370`

```rust
fn parse_text_mode(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    // 1. Check size limit (64 KiB)
    if input.len() > MAX_TEXT_SIZE {
        return Err(nom::Err::Failure(...));
    }

    // 2. Validate UTF-8
    if std::str::from_utf8(input).is_err() {
        return Err(nom::Err::Failure(...));
    }

    // 3. Check for prohibited control chars
    for &byte in input {
        match byte {
            b'\t' | b'\n' | b'\r' => continue,  // Allowed
            0x00..=0x08 | 0x0B..=0x1F | 0x7F => {
                return Err(nom::Err::Failure(...));
            }
            _ => continue,
        }
    }

    // Valid! Return as-is
    Ok((&b""[..], input.to_vec()))
}
```

**Validation rules:**

- ✅ Valid UTF-8
- ✅ ≤ 64 KiB
- ✅ Only allows `\t`, `\n`, `\r` control chars
- ❌ Rejects `\0`, `\b`, `\f`, `\v`, etc.

**Example:**

```rust
parse_text_mode(b"Hello\nWorld");    // ✅ OK
parse_text_mode(b"Hello\x00World");  // ❌ Null byte rejected
parse_text_mode(b"Hello\x0CWorld");  // ❌ Form feed rejected
```

---

### 3. `parse_binary_mode` - Binary Decoder

**Location:** `src/lib.rs:324-332`

```rust
fn parse_binary_mode(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (input, chunks) = many0(parse_binary_chunk)(input)?;
    let result = chunks.into_iter().flatten().collect();
    Ok((input, result))
}
```

**How it works:**

1. Uses `many0` to parse zero or more chunks
2. Each chunk is parsed by `parse_binary_chunk`
3. Flattens all chunks into a single byte vector

**What's a chunk?** One of:

- **Z85 block** (5 chars → 4 bytes)
- **Single raw** (`|xxxx` → 4 bytes)
- **Multi raw** (`N|xxxx...` → N×4 bytes)
- **Terminal raw** (`||...` → rest of data)

---

### 4. `parse_binary_chunk` - Chunk Parser

**Location:** `src/lib.rs:314-322`

```rust
fn parse_binary_chunk(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    alt((
        parse_terminal_raw_block,  // Try terminal first
        parse_multi_raw_block,     // Then multi-block
        parse_single_raw_block,    // Then single block
        parse_z85_block,           // Finally Z85
    ))(input)
}
```

**The `alt` combinator:**

- Tries each parser in order
- Returns the first successful match
- Order matters! Terminal must come before multi (both start with `|`)

**Example flow:**

```
Input: "|Test"
  ├─ parse_terminal_raw_block → Fails (needs "||")
  ├─ parse_multi_raw_block → Fails (needs digit before |)
  ├─ parse_single_raw_block → SUCCESS! Parses "|Test"
  └─ parse_z85_block → Not tried (already succeeded)
```

---

## Composition Patterns

### Pattern 1: Sequential Parsing

Parse multiple parts in sequence using `tuple`:

```rust
use nom::sequence::tuple;

fn parse_header_and_jeb85(input: &[u8]) -> IResult<&[u8], (Vec<u8>, Vec<u8>)> {
    tuple((
        parse_header,      // Parse header first
        parse_jeb85,       // Then parse JEB85 data
    ))(input)
}
```

### Pattern 2: Alternative Parsing

Try different parsers using `alt`:

```rust
use nom::branch::alt;

fn parse_any_encoding(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    alt((
        parse_base64,      // Try base64 first
        parse_jeb85,       // Then JEB85
        parse_hex,         // Finally hex
    ))(input)
}
```

### Pattern 3: Repeated Parsing

Parse multiple items using `many0` or `many1`:

```rust
use nom::multi::many0;

fn parse_jeb85_list(input: &[u8]) -> IResult<&[u8], Vec<Vec<u8>>> {
    many0(|input| {
        let (input, data) = parse_jeb85(input)?;
        let (input, _) = tag(b",")(input)?;  // Separator
        Ok((input, data))
    })(input)
}
```

### Pattern 4: Optional Parsing

Parse optional parts using `opt`:

```rust
use nom::combinator::opt;

fn parse_with_optional_jeb85(input: &[u8]) -> IResult<&[u8], Option<Vec<u8>>> {
    opt(parse_jeb85)(input)
}
```

### Pattern 5: Conditional Parsing

Parse based on a condition:

```rust
fn parse_conditional(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    let (input, mode) = nom::bytes::complete::take(1usize)(input)?;

    match mode[0] {
        b'T' => parse_text_mode(input),
        b'B' => parse_binary_mode(input),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}
```

---

## Advanced Usage

### Building a Protocol Parser

Combine JEB85 with other combinators to build a protocol:

```rust
use nom::{
    bytes::complete::{tag, take},
    number::complete::be_u32,
    sequence::{preceded, tuple},
    IResult,
};

// Protocol format: [MAGIC:4][LENGTH:4][DATA:LENGTH]
fn parse_protocol_message(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    // Parse magic bytes
    let (input, _) = tag(b"JEB!")(input)?;

    // Parse length (big-endian u32)
    let (input, length) = be_u32(input)?;

    // Take exactly 'length' bytes
    let (input, data_bytes) = take(length as usize)(input)?;

    // Decode as JEB85
    let data_str = std::str::from_utf8(data_bytes)
        .map_err(|_| nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Char,
        )))?;

    let decoded = json_encoded_binary::decode(data_str)
        .map_err(|_| nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Verify,
        )))?;

    Ok((input, decoded))
}
```

### Streaming Parser

Parse JEB85 data from a stream:

```rust
use nom::Needed;

fn parse_jeb85_streaming(input: &[u8]) -> nom::IResult<&[u8], Vec<u8>> {
    // In streaming mode, nom can indicate more data is needed
    match parse_jeb85(input) {
        Ok(result) => Ok(result),
        Err(nom::Err::Incomplete(needed)) => {
            // Tell caller how much more data is needed
            Err(nom::Err::Incomplete(needed))
        }
        Err(e) => Err(e),
    }
}
```

### Custom Error Handling

Add custom error types:

```rust
use nom::error::{Error, ErrorKind, ParseError};

#[derive(Debug, PartialEq)]
enum CustomError {
    InvalidJeb85,
    TooLarge,
    InvalidUtf8,
}

fn parse_with_custom_errors(input: &str) -> Result<Vec<u8>, CustomError> {
    json_encoded_binary::decode(input).map_err(|e| match e {
        json_encoded_binary::Jeb85Error::TextTooLarge => CustomError::TooLarge,
        json_encoded_binary::Jeb85Error::InvalidUtf8 => CustomError::InvalidUtf8,
        _ => CustomError::InvalidJeb85,
    })
}
```

---

## Key Takeaways

1. **Combinators are composable**: Build complex parsers from simple ones
2. **Order matters in `alt`**: First match wins
3. **Error propagation**: Use `?` to propagate errors up the chain
4. **IResult structure**: Always `(remaining_input, parsed_value)`
5. **The split architecture** enables:
   - Independent testing of each component
   - Clear separation of concerns
   - Easy composition with other parsers
   - Better error messages

## Further Reading

- [Nom Documentation](https://docs.rs/nom/)
- [Nom Recipes](https://github.com/rust-bakery/nom/blob/main/doc/nom_recipes.md)
- [Parser Combinators Explained](https://bodil.lol/parser-combinators/)
