use {serde_json::Value, std::collections::HashMap};

#[derive(Debug, Clone, PartialEq)]
pub enum ParseError {
    UnexpectedEof,
    UnexpectedChar(char),
    InvalidNumber(String),
    InvalidEscape,
    UnterminatedString,
    UnterminatedComment,
    ExpectedColon,
    ExpectedValue,
    MultipleCommas,
    InvalidIdentifier(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::UnexpectedEof => write!(f, "Unexpected end of input"),
            ParseError::UnexpectedChar(c) => write!(f, "Unexpected character: {}", c),
            ParseError::InvalidNumber(s) => write!(f, "Invalid number: {}", s),
            ParseError::InvalidEscape => write!(f, "Invalid escape sequence"),
            ParseError::UnterminatedString => write!(f, "Unterminated string"),
            ParseError::UnterminatedComment => write!(f, "Unterminated block comment"),
            ParseError::ExpectedColon => write!(f, "Expected ':' after object key"),
            ParseError::ExpectedValue => write!(f, "Expected value"),
            ParseError::MultipleCommas => write!(f, "Multiple consecutive commas not allowed"),
            ParseError::InvalidIdentifier(s) => write!(f, "Invalid identifier: {}", s),
        }
    }
}

impl std::error::Error for ParseError {}

pub type Result<T> = std::result::Result<T, ParseError>;

struct Lexer<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Lexer<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    fn current_char(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.current_char()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    fn peek_char(&self, offset: usize) -> Option<char> {
        self.input[self.pos..].chars().nth(offset)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn skip_line_comment(&mut self) {
        // Skip // or # style comment
        while let Some(c) = self.current_char() {
            self.advance();
            if c == '\n' {
                break;
            }
        }
    }

    fn skip_block_comment(&mut self) -> Result<()> {
        // Skip /* */ style comment
        // We're positioned at '/'
        self.advance(); // skip '/'
        self.advance(); // skip '*'

        loop {
            match self.current_char() {
                None => return Err(ParseError::UnterminatedComment),
                Some('*') => {
                    self.advance();
                    if self.current_char() == Some('/') {
                        self.advance();
                        return Ok(());
                    }
                }
                Some(_) => {
                    self.advance();
                }
            }
        }
    }

    fn skip_whitespace_and_comments(&mut self) -> Result<()> {
        loop {
            self.skip_whitespace();
            match self.current_char() {
                Some('/') => {
                    if self.peek_char(1) == Some('/') {
                        self.skip_line_comment();
                    } else if self.peek_char(1) == Some('*') {
                        self.skip_block_comment()?;
                    } else {
                        break;
                    }
                }
                Some('#') => {
                    self.skip_line_comment();
                }
                _ => break,
            }
        }
        Ok(())
    }

    fn parse_string(&mut self) -> Result<String> {
        // We're at opening quote
        self.advance(); // skip opening quote

        let mut result = String::new();

        loop {
            match self.current_char() {
                None => return Err(ParseError::UnterminatedString),
                Some('"') => {
                    self.advance(); // skip closing quote
                    return Ok(result);
                }
                Some('\\') => {
                    self.advance();
                    match self.current_char() {
                        None => return Err(ParseError::InvalidEscape),
                        Some('n') => {
                            result.push('\n');
                            self.advance();
                        }
                        Some('r') => {
                            result.push('\r');
                            self.advance();
                        }
                        Some('t') => {
                            result.push('\t');
                            self.advance();
                        }
                        Some('\\') => {
                            result.push('\\');
                            self.advance();
                        }
                        Some('"') => {
                            result.push('"');
                            self.advance();
                        }
                        Some('/') => {
                            result.push('/');
                            self.advance();
                        }
                        Some('b') => {
                            result.push('\u{0008}');
                            self.advance();
                        }
                        Some('f') => {
                            result.push('\u{000C}');
                            self.advance();
                        }
                        Some('u') => {
                            self.advance();
                            let mut hex = String::new();
                            for _ in 0..4 {
                                match self.current_char() {
                                    Some(c) if c.is_ascii_hexdigit() => {
                                        hex.push(c);
                                        self.advance();
                                    }
                                    _ => return Err(ParseError::InvalidEscape),
                                }
                            }
                            let code = u32::from_str_radix(&hex, 16)
                                .map_err(|_| ParseError::InvalidEscape)?;
                            let ch = char::from_u32(code).ok_or(ParseError::InvalidEscape)?;
                            result.push(ch);
                        }
                        Some(c) => return Err(ParseError::UnexpectedChar(c)),
                    }
                }
                Some(c) => {
                    result.push(c);
                    self.advance();
                }
            }
        }
    }

    fn parse_number(&mut self) -> Result<Value> {
        let start = self.pos;

        // Handle negative sign
        if self.current_char() == Some('-') {
            self.advance();
        }

        // Parse integer part
        match self.current_char() {
            Some('0') => {
                self.advance();
            }
            Some(c) if c.is_ascii_digit() => {
                while let Some(c) = self.current_char() {
                    if c.is_ascii_digit() {
                        self.advance();
                    } else {
                        break;
                    }
                }
            }
            _ => return Err(ParseError::ExpectedValue),
        }

        // Parse decimal part
        if self.current_char() == Some('.') {
            self.advance();
            if !matches!(self.current_char(), Some(c) if c.is_ascii_digit()) {
                return Err(ParseError::InvalidNumber(
                    self.input[start..self.pos].to_string(),
                ));
            }
            while let Some(c) = self.current_char() {
                if c.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        // Parse exponent part
        if matches!(self.current_char(), Some('e') | Some('E')) {
            self.advance();
            if matches!(self.current_char(), Some('+') | Some('-')) {
                self.advance();
            }
            if !matches!(self.current_char(), Some(c) if c.is_ascii_digit()) {
                return Err(ParseError::InvalidNumber(
                    self.input[start..self.pos].to_string(),
                ));
            }
            while let Some(c) = self.current_char() {
                if c.is_ascii_digit() {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        let num_str = &self.input[start..self.pos];

        // Try to parse as i64 first
        if let Ok(i) = num_str.parse::<i64>() {
            return Ok(Value::Number(i.into()));
        }

        // Otherwise parse as f64
        let f = num_str
            .parse::<f64>()
            .map_err(|_| ParseError::InvalidNumber(num_str.to_string()))?;

        Ok(Value::Number(serde_json::Number::from_f64(f).ok_or_else(
            || ParseError::InvalidNumber(num_str.to_string()),
        )?))
    }

    fn parse_identifier(&mut self) -> Result<String> {
        let start = self.pos;

        // First character: [a-zA-Z_]
        match self.current_char() {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => {
                self.advance();
            }
            Some(c) => return Err(ParseError::InvalidIdentifier(c.to_string())),
            None => return Err(ParseError::UnexpectedEof),
        }

        // Subsequent characters: [a-zA-Z0-9_]
        while let Some(c) = self.current_char() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }

        Ok(self.input[start..self.pos].to_string())
    }

    fn parse_value(&mut self) -> Result<Value> {
        self.skip_whitespace_and_comments()?;

        match self.current_char() {
            None => Err(ParseError::UnexpectedEof),
            Some('"') => Ok(Value::String(self.parse_string()?)),
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('t') => {
                if self.input[self.pos..].starts_with("true") {
                    self.pos += 4;
                    Ok(Value::Bool(true))
                } else {
                    Err(ParseError::ExpectedValue)
                }
            }
            Some('f') => {
                if self.input[self.pos..].starts_with("false") {
                    self.pos += 5;
                    Ok(Value::Bool(false))
                } else {
                    Err(ParseError::ExpectedValue)
                }
            }
            Some('n') => {
                if self.input[self.pos..].starts_with("null") {
                    self.pos += 4;
                    Ok(Value::Null)
                } else {
                    Err(ParseError::ExpectedValue)
                }
            }
            Some(c) if c == '-' || c.is_ascii_digit() => self.parse_number(),
            Some(c) => Err(ParseError::UnexpectedChar(c)),
        }
    }

    fn parse_array(&mut self) -> Result<Value> {
        self.advance(); // skip '['

        let mut items = Vec::new();
        let mut expect_comma = false;
        let mut saw_comma = false;

        loop {
            self.skip_whitespace_and_comments()?;

            // Check for closing bracket
            if self.current_char() == Some(']') {
                self.advance();
                return Ok(Value::Array(items));
            }

            // Check for comma
            if self.current_char() == Some(',') {
                if saw_comma && expect_comma {
                    // Multiple commas in a row
                    return Err(ParseError::MultipleCommas);
                }
                if items.is_empty() {
                    // Leading comma
                    return Err(ParseError::MultipleCommas);
                }
                if !expect_comma && !items.is_empty() {
                    // Comma before first element (after other elements were added)
                    return Err(ParseError::MultipleCommas);
                }
                self.advance();
                saw_comma = true;
                self.skip_whitespace_and_comments()?;

                // Check for trailing comma
                if self.current_char() == Some(']') {
                    self.advance();
                    return Ok(Value::Array(items));
                }

                expect_comma = false;
                continue;
            }

            if expect_comma && !saw_comma && !items.is_empty() {
                // We need either a comma or closing bracket
                // If we're here, we have neither, but we already checked for
                // ']' above So this must be a value without a
                // separator This is allowed in our lenient
                // format
            }

            // Parse value
            let value = self.parse_value()?;
            items.push(value);
            expect_comma = true;
            saw_comma = false;
        }
    }

    fn parse_object(&mut self) -> Result<Value> {
        self.advance(); // skip '{'

        let mut map = HashMap::new();
        let mut expect_comma = false;
        let mut saw_comma = false;

        loop {
            self.skip_whitespace_and_comments()?;

            // Check for closing brace
            if self.current_char() == Some('}') {
                self.advance();
                return Ok(Value::Object(map.into_iter().collect()));
            }

            // Check for comma
            if self.current_char() == Some(',') {
                if saw_comma && expect_comma {
                    return Err(ParseError::MultipleCommas);
                }
                if map.is_empty() {
                    // Leading comma
                    return Err(ParseError::MultipleCommas);
                }
                if !expect_comma && !map.is_empty() {
                    return Err(ParseError::MultipleCommas);
                }
                self.advance();
                saw_comma = true;
                self.skip_whitespace_and_comments()?;

                // Check for trailing comma
                if self.current_char() == Some('}') {
                    self.advance();
                    return Ok(Value::Object(map.into_iter().collect()));
                }

                expect_comma = false;
                continue;
            }

            if expect_comma && !saw_comma && !map.is_empty() {
                // Value without separator - allowed in lenient format
            }

            // Parse key
            let key = match self.current_char() {
                Some('"') => self.parse_string()?,
                Some(c) if c.is_ascii_alphabetic() || c == '_' => self.parse_identifier()?,
                Some(c) => return Err(ParseError::UnexpectedChar(c)),
                None => return Err(ParseError::UnexpectedEof),
            };

            self.skip_whitespace_and_comments()?;

            // Expect colon
            if self.current_char() != Some(':') {
                return Err(ParseError::ExpectedColon);
            }
            self.advance(); // skip ':'

            self.skip_whitespace_and_comments()?;

            // Parse value
            let value = self.parse_value()?;
            map.insert(key, value);
            expect_comma = true;
            saw_comma = false;
        }
    }

    fn remainder(&self) -> &'a str {
        &self.input[self.pos..]
    }
}

/// Parse the first JSON value from the input and return it along with the
/// unparsed remainder.
///
/// # Examples
///
/// ```
/// use slop_lenient_json::parse_first;
///
/// let input = r#"{"name": "Alice"} {"name": "Bob"}"#;
/// let (value, remainder) = parse_first(input).unwrap();
/// assert_eq!(value["name"], "Alice");
/// assert_eq!(remainder.trim(), r#"{"name": "Bob"}"#);
/// ```
pub fn parse_first(input: &str) -> Result<(Value, &str)> {
    let mut lexer = Lexer::new(input);
    let value = lexer.parse_value()?;
    let remainder = lexer.remainder();
    Ok((value, remainder))
}

/// Parse a complete JSON value from the input, expecting no trailing content.
///
/// # Examples
///
/// ```
/// use slop_lenient_json::parse;
///
/// let value = parse(r#"{name: "Alice", age: 30}"#).unwrap();
/// assert_eq!(value["name"], "Alice");
/// assert_eq!(value["age"], 30);
/// ```
pub fn parse(input: &str) -> Result<Value> {
    let (value, remainder) = parse_first(input)?;
    let remainder = remainder.trim();
    if !remainder.is_empty() {
        return Err(ParseError::UnexpectedChar(
            remainder.chars().next().unwrap(),
        ));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_object() {
        let input = r#"{"name": "Alice", "age": 30}"#;
        let value = parse(input).unwrap();
        assert_eq!(value["name"], "Alice");
        assert_eq!(value["age"], 30);
    }

    #[test]
    fn test_basic_array() {
        let input = r#"[1, 2, 3]"#;
        let value = parse(input).unwrap();
        assert_eq!(value, serde_json::json!([1, 2, 3]));
    }

    #[test]
    fn test_unquoted_keys() {
        let input = r#"{name: "Alice", age: 30}"#;
        let value = parse(input).unwrap();
        assert_eq!(value["name"], "Alice");
        assert_eq!(value["age"], 30);
    }

    #[test]
    fn test_unquoted_keys_with_underscores() {
        let input = r#"{user_name: "Alice", user_id: 123}"#;
        let value = parse(input).unwrap();
        assert_eq!(value["user_name"], "Alice");
        assert_eq!(value["user_id"], 123);
    }

    #[test]
    fn test_optional_commas_array() {
        let input = r#"[1 2 3]"#;
        let value = parse(input).unwrap();
        assert_eq!(value, serde_json::json!([1, 2, 3]));
    }

    #[test]
    fn test_optional_commas_object() {
        let input = r#"{a: 1 b: 2}"#;
        let value = parse(input).unwrap();
        assert_eq!(value["a"], 1);
        assert_eq!(value["b"], 2);
    }

    #[test]
    fn test_trailing_comma_array() {
        let input = r#"[1, 2, 3,]"#;
        let value = parse(input).unwrap();
        assert_eq!(value, serde_json::json!([1, 2, 3]));
    }

    #[test]
    fn test_trailing_comma_object() {
        let input = r#"{a: 1, b: 2,}"#;
        let value = parse(input).unwrap();
        assert_eq!(value["a"], 1);
        assert_eq!(value["b"], 2);
    }

    #[test]
    fn test_mixed_commas() {
        let input = r#"[1, 2 3, 4]"#;
        let value = parse(input).unwrap();
        assert_eq!(value, serde_json::json!([1, 2, 3, 4]));
    }

    #[test]
    fn test_line_comment_slash() {
        let input = r#"{
            // This is a comment
            name: "Alice"
        }"#;
        let value = parse(input).unwrap();
        assert_eq!(value["name"], "Alice");
    }

    #[test]
    fn test_line_comment_hash() {
        let input = r#"{
            # This is a comment
            name: "Alice"
        }"#;
        let value = parse(input).unwrap();
        assert_eq!(value["name"], "Alice");
    }

    #[test]
    fn test_block_comment() {
        let input = r#"{
            /* This is a
               multi-line comment */
            name: "Alice"
        }"#;
        let value = parse(input).unwrap();
        assert_eq!(value["name"], "Alice");
    }

    #[test]
    fn test_multiline_string() {
        let input = r#"{"message": "Hello
World"}"#;
        let value = parse(input).unwrap();
        assert_eq!(value["message"], "Hello\nWorld");
    }

    #[test]
    fn test_streaming_parse() {
        let input = r#"{"name": "Alice"} {"name": "Bob"}"#;
        let (value1, remainder) = parse_first(input).unwrap();
        assert_eq!(value1["name"], "Alice");

        let (value2, remainder) = parse_first(remainder).unwrap();
        assert_eq!(value2["name"], "Bob");
        assert_eq!(remainder.trim(), "");
    }

    #[test]
    fn test_multiple_commas_rejected() {
        let input = r#"[1,, 2]"#;
        assert!(parse(input).is_err());
    }

    #[test]
    fn test_nested_structures() {
        let input = r#"{
            users: [
                {name: "Alice", age: 30},
                {name: "Bob", age: 25}
            ]
        }"#;
        let value = parse(input).unwrap();
        assert_eq!(value["users"][0]["name"], "Alice");
        assert_eq!(value["users"][1]["name"], "Bob");
    }

    #[test]
    fn test_all_value_types() {
        let input = r#"{
            string: "hello",
            number: 42,
            float: 3.14,
            bool_true: true,
            bool_false: false,
            null_value: null,
            array: [1, 2, 3],
            object: {nested: "value"}
        }"#;
        let value = parse(input).unwrap();
        assert_eq!(value["string"], "hello");
        assert_eq!(value["number"], 42);
        assert_eq!(value["float"], 3.14);
        assert_eq!(value["bool_true"], true);
        assert_eq!(value["bool_false"], false);
        assert_eq!(value["null_value"], serde_json::Value::Null);
        assert_eq!(value["array"], serde_json::json!([1, 2, 3]));
        assert_eq!(value["object"]["nested"], "value");
    }

    #[test]
    fn test_complex_example() {
        let input = r#"{
            // Server configuration
            server: {
                host: "localhost"
                port: 8080,
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
        }"#;
        let value = parse(input).unwrap();
        assert_eq!(value["server"]["host"], "localhost");
        assert_eq!(value["server"]["port"], 8080);
        assert_eq!(value["database"]["url"], "postgresql://localhost/mydb");
        assert_eq!(value["features"]["enable_cache"], true);
    }

    #[test]
    fn test_escape_sequences() {
        let input = r#"{"message": "Hello\nWorld\t!"}"#;
        let value = parse(input).unwrap();
        assert_eq!(value["message"], "Hello\nWorld\t!");
    }

    #[test]
    fn test_unicode_escape() {
        let input = r#"{"emoji": "\u0048\u0065\u006C\u006C\u006F"}"#;
        let value = parse(input).unwrap();
        assert_eq!(value["emoji"], "Hello");
    }

    #[test]
    fn test_negative_numbers() {
        let input = r#"[-1, -3.14, -0]"#;
        let value = parse(input).unwrap();
        assert_eq!(value[0], -1);
        assert_eq!(value[1], -3.14);
        assert_eq!(value[2], 0);
    }

    #[test]
    fn test_scientific_notation() {
        let input = r#"[1e10, 1.5e-5, 2E+3]"#;
        let value = parse(input).unwrap();
        assert_eq!(value[0], 1e10);
        assert_eq!(value[1], 1.5e-5);
        assert_eq!(value[2], 2e3);
    }

    #[test]
    fn test_empty_structures() {
        let input = r#"{}"#;
        let value = parse(input).unwrap();
        assert!(value.as_object().unwrap().is_empty());

        let input = r#"[]"#;
        let value = parse(input).unwrap();
        assert!(value.as_array().unwrap().is_empty());
    }

    #[test]
    fn test_whitespace_handling() {
        let input = "  \n\t  {  \n  a  :  1  \n  }  \n  ";
        let value = parse(input).unwrap();
        assert_eq!(value["a"], 1);
    }

    // Additional edge case tests

    #[test]
    fn test_comments_in_array() {
        let input = r#"[
            1, // first element
            2  # second element
            /* third */ 3
        ]"#;
        let value = parse(input).unwrap();
        assert_eq!(value, serde_json::json!([1, 2, 3]));
    }

    #[test]
    fn test_comments_between_fields() {
        let input = r#"{
            a: 1 // comment after value
            # comment before key
            b: 2
        }"#;
        let value = parse(input).unwrap();
        assert_eq!(value["a"], 1);
        assert_eq!(value["b"], 2);
    }

    #[test]
    fn test_unquoted_key_starting_with_underscore() {
        let input = r#"{_private: "secret", __dunder: true}"#;
        let value = parse(input).unwrap();
        assert_eq!(value["_private"], "secret");
        assert_eq!(value["__dunder"], true);
    }

    #[test]
    fn test_mixed_quoted_unquoted_keys() {
        let input = r#"{
            name: "Alice",
            "age": 30,
            user_id: 123,
            "user-type": "admin"
        }"#;
        let value = parse(input).unwrap();
        assert_eq!(value["name"], "Alice");
        assert_eq!(value["age"], 30);
        assert_eq!(value["user_id"], 123);
        assert_eq!(value["user-type"], "admin");
    }

    #[test]
    fn test_empty_string() {
        let input = r#"{name: "", value: ""}"#;
        let value = parse(input).unwrap();
        assert_eq!(value["name"], "");
        assert_eq!(value["value"], "");
    }

    #[test]
    fn test_all_escape_sequences() {
        let input = r#"{"test": "quote: \" backslash: \\ slash: \/ newline: \n tab: \t"}"#;
        let value = parse(input).unwrap();
        assert_eq!(
            value["test"],
            "quote: \" backslash: \\ slash: / newline: \n tab: \t"
        );
    }

    #[test]
    fn test_leading_comma_rejected_array() {
        let input = r#"[, 1, 2]"#;
        assert!(parse(input).is_err());
    }

    #[test]
    fn test_leading_comma_rejected_object() {
        let input = r#"{, a: 1}"#;
        assert!(parse(input).is_err());
    }

    #[test]
    fn test_deeply_nested() {
        let input = r#"{
            level1: {
                level2: {
                    level3: {
                        level4: {
                            value: "deep"
                        }
                    }
                }
            }
        }"#;
        let value = parse(input).unwrap();
        assert_eq!(
            value["level1"]["level2"]["level3"]["level4"]["value"],
            "deep"
        );
    }

    #[test]
    fn test_array_of_objects_with_comments() {
        let input = r#"[
            // First user
            { name: "Alice" age: 30 }
            // Second user
            { name: "Bob", age: 25 },
        ]"#;
        let value = parse(input).unwrap();
        assert_eq!(value[0]["name"], "Alice");
        assert_eq!(value[0]["age"], 30);
        assert_eq!(value[1]["name"], "Bob");
        assert_eq!(value[1]["age"], 25);
    }

    #[test]
    fn test_number_zero_variants() {
        let input = r#"[0, 0.0, 0e0, 0.0e0]"#;
        let value = parse(input).unwrap();
        assert_eq!(value[0], 0);
        assert_eq!(value[1], 0.0);
        assert_eq!(value[2], 0.0);
        assert_eq!(value[3], 0.0);
    }

    #[test]
    fn test_large_integers() {
        let input = r#"[9007199254740991, -9007199254740991]"#;
        let value = parse(input).unwrap();
        assert_eq!(value[0], 9007199254740991_i64);
        assert_eq!(value[1], -9007199254740991_i64);
    }

    #[test]
    fn test_streaming_with_newlines() {
        let input = "{\n  name: \"Alice\"\n}\n{\n  name: \"Bob\"\n}";
        let (value1, remainder) = parse_first(input).unwrap();
        assert_eq!(value1["name"], "Alice");

        let (value2, remainder) = parse_first(remainder).unwrap();
        assert_eq!(value2["name"], "Bob");
        assert_eq!(remainder.trim(), "");
    }

    #[test]
    fn test_multiline_string_with_escapes() {
        let input = r#"{"text": "Line 1\nLine 2
Line 3"}"#;
        let value = parse(input).unwrap();
        assert_eq!(value["text"], "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_object_with_only_trailing_comma() {
        let input = r#"{a: 1,}"#;
        let value = parse(input).unwrap();
        assert_eq!(value["a"], 1);
    }

    #[test]
    fn test_array_with_only_trailing_comma() {
        let input = r#"[1,]"#;
        let value = parse(input).unwrap();
        assert_eq!(value, serde_json::json!([1]));
    }

    #[test]
    fn test_empty_array_with_comment() {
        let input = r#"[/* empty */]"#;
        let value = parse(input).unwrap();
        assert!(value.as_array().unwrap().is_empty());
    }

    #[test]
    fn test_empty_object_with_comment() {
        let input = r#"{// nothing here
        }"#;
        let value = parse(input).unwrap();
        assert!(value.as_object().unwrap().is_empty());
    }
}
