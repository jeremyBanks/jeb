# JEB Lenient JSON Specification

A more forgiving JSON variant designed to handle common parsing errors and
hand-edited configuration files, while remaining close enough to standard JSON
to be easily understood.

## Design Goals

- Handle common typos and formatting variations gracefully
- Support human-friendly features (comments, unquoted keys)
- Enable streaming/incremental parsing (read first value, get remainder)
- Stay within the realm of "plausible errors" rather than "total absurdity"

## Features

### 1. Flexible Comma Handling

Commas between array elements and object fields are **optional**, and trailing
commas are allowed.

**Rules:**

- Between items: 0 or 1 comma
- Trailing: 0 or 1 comma
- Multiple consecutive commas are **not allowed** (this would be absurd)

**Valid:**

```json
[1, 2, 3]
[1 2 3]
[1, 2, 3,]
[1 2 3,]
{a: 1, b: 2}
{a: 1 b: 2}
{a: 1, b: 2,}
```

**Invalid:**

```json
[1,, 2]      // double comma
[1,,,]       // multiple trailing commas
{a: 1,, b: 2}
```

### 2. Comments

Three comment styles are supported:

**Line comments:** `//` (JavaScript-style)

```json
{
  // This is a comment
  "name": "value"
}
```

**Shell comments:** `#` (Python/Shell-style)

```json
{
  # This is also a comment
  name: "value"
}
```

**Block comments:** `/* */` (C-style)

```json
{
  /* This is a
     multi-line comment */
  "name": "value"
}
```

### 3. Unquoted Object Keys

Object keys can be written without quotes if they match JavaScript identifier
rules (or a subset thereof).

**Rules for unquoted keys:**

- Must start with: `[a-zA-Z_]`
- Can contain: `[a-zA-Z0-9_]`
- Hyphens are **not allowed** (not valid JavaScript identifiers)

**Valid:**

```json
{
  "name": "Alice",
  "user_id": 123,
  "isActive": true,
  "_private": false
}
```

**Invalid:**

```json
{
  user-id: 123,     // hyphen not allowed
  123abc: "value",  // can't start with digit
  $special: "value" // $ not in our subset (though valid in JS)
}
```

**Note:** Keys that don't match these rules must still be quoted:

```json
{
  "user-id": 123,
  "123": "numeric key",
  "with spaces": "quoted"
}
```

### 4. Multiline String Literals

String literals can contain literal newline characters without requiring `\n`
escape sequences.

**Valid:**

```json
{
  "message": "Hello
World
Here"
}
```

This is equivalent to standard JSON:

```json
{
  "message": "Hello\nWorld\nHere"
}
```

**Note:** Standard escape sequences still work:

- `\"` for quote
- `\\` for backslash
- `\n` for newline (redundant but allowed)
- `\t` for tab
- etc.

## Parsing Requirements

### Streaming / Incremental Parsing

The parser must support reading a single value from the beginning of an input
string and returning:

1. The parsed value
2. A slice/reference to the remainder of the input (unparsed portion)

This is similar to `serde_json`'s streaming deserializer functionality.

**Example:**

```rust
let input = r#"{"name": "Alice"} {"name": "Bob"}"#;
let (value1, remainder) = parse_first(input)?;
// value1 = {"name": "Alice"}
// remainder = r#" {"name": "Bob"}"#

let (value2, remainder) = parse_first(remainder)?;
// value2 = {"name": "Bob"}
// remainder = ""
```

This enables:

- Parsing newline-delimited JSON (NDJSON/JSON Lines)
- Processing streams without loading everything into memory
- Reading multiple top-level values from a single input

## Compatibility Notes

### Strict Subset of Standard JSON

Any **valid standard JSON** is also valid JEB Lenient JSON (with identical
semantics).

### Not Compatible With

This format intentionally does **not** support:

- Hexadecimal numbers (`0xFF`)
- Binary literals (`0b1010`)
- Special numeric values (`NaN`, `Infinity`, `-Infinity`)
- Unquoted string values (only keys can be unquoted)
- Single-quoted strings (only double quotes)

These features, while present in some JSON variants (like JSON5), are omitted to
keep the format simple and close to standard JSON.

## Rationale

### Why These Features?

**Flexible commas:** The most common JSON syntax errors involve missing or
trailing commas. Making commas optional (but not absurd) handles both issues
elegantly.

**Comments:** Essential for configuration files and hand-edited data. Three
styles accommodate different communities (JS, shell, C).

**Unquoted keys:** Reduces visual clutter in configuration files while
maintaining clear structure. Limited to simple identifiers to avoid ambiguity.

**Multiline strings:** Natural for embedding longer text, error messages, or
documentation without escape sequence soup.

### Why Not Full "Commas as Whitespace"?

Treating commas as arbitrary whitespace (allowing `[1,,,,,2]`) crosses from
"handling errors gracefully" into "encouraging bad practices." The 0-or-1 rule
handles realistic typos without becoming absurd.

### Why Not More Features?

Features like hex literals, unquoted string values, or single-quoted strings add
complexity and move further from standard JSON. The goal is to be "lenient" not
"a different language."

## Examples

### Configuration File

```json
{
  // Server configuration
  server: {
    host: "localhost"
    port: 8080,  // trailing comma ok
    max_connections: 100
  },

  # Database settings
  database: {
    url: "postgresql://localhost/mydb",
    pool_size: 10
  }

  /* Feature flags */
  features: {
    enable_cache: true,
    enable_logging: true,
  }
}
```

### Newline-Delimited Stream

```json
{name: "Alice", age: 30}
{name: "Bob", age: 25}
{name: "Charlie" age: 35,}
```

Each line is independently parseable with the streaming API.

## Implementation Notes

- Whitespace (space, tab, newline, carriage return) is still used to separate
  tokens
- Commas are treated as optional separators, not as whitespace themselves
- Parser should track line/column numbers for error reporting
- Comments should be skipped during tokenization
- Unquoted keys should be tokenized separately from other identifiers (`true`,
  `false`, `null`)
