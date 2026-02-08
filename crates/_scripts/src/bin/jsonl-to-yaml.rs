use {
    anyhow::{
        Context,
        Result,
    },
    serde_json::Value,
    std::io::{
        self,
        BufRead,
        Write,
    },
};

fn main() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    let mut wrote_any = false;

    for (line_num, line) in stdin.lock().lines().enumerate() {
        let line = line.with_context(|| format!("Failed to read line {}", line_num + 1))?;

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        let value: Value = serde_json::from_str(&line)
            .with_context(|| format!("Failed to parse JSON on line {}", line_num + 1))?;

        writeln!(stdout, "---")?;
        write_yaml_value(&mut stdout, &value, 0, false)?;
        writeln!(stdout, "...")?;
        wrote_any = true;
    }

    // Always end with a blank line at EOF
    if wrote_any {
        writeln!(stdout)?;
    }

    Ok(())
}

/// Check if a string can be represented unquoted in YAML
fn can_be_unquoted(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // Check if it matches the allowed unquoted pattern:
    // /^[_$a-zA-Z]([_$a-zA-Z0-9\-. ]*[_$a-zA-Z0-9\-.])?$/
    let chars: Vec<char> = s.chars().collect();

    // First character must be [_$a-zA-Z]
    let first = chars[0];
    if !matches!(first, '_' | '$' | 'a'..='z' | 'A'..='Z') {
        return false;
    }

    if chars.len() == 1 {
        // Single character matching first char class is OK, unless it's a reserved word
        return !is_yaml_reserved(s);
    }

    // Last character must be [_$a-zA-Z0-9\-.]
    let last = *chars.last().unwrap();
    if !matches!(last, '_' | '$' | 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '.') {
        return false;
    }

    // Middle characters must be [_$a-zA-Z0-9\-. ] (including space)
    for &c in &chars[1..chars.len() - 1] {
        if !matches!(c, '_' | '$' | 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '.' | ' ') {
            return false;
        }
    }

    // Check it's not a reserved word
    !is_yaml_reserved(s)
}

/// Check if a string is a YAML reserved word (case-insensitive)
fn is_yaml_reserved(s: &str) -> bool {
    matches!(
        s.to_lowercase().as_str(),
        "null" | "~" | "true" | "false" | "yes" | "no" | "on" | "off" | "y" | "n" | "t" | "f"
    )
}

/// Escape a string for double-quoted YAML (using JSON escape sequences)
fn escape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len() + 2);
    result.push('"');

    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            '\x08' => result.push_str("\\b"),
            '\x0c' => result.push_str("\\f"),
            c if c.is_control() => {
                // Use \uXXXX for other control characters
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }

    result.push('"');
    result
}

/// Write a YAML string value with appropriate formatting
fn write_yaml_string<W: Write>(w: &mut W, s: &str, indent: usize) -> Result<()> {
    if s.contains('\n') {
        // Multi-line string: use literal block scalar
        // Count trailing newlines to choose the right chomping indicator:
        // - `|-` (strip): no trailing newlines
        // - `|` (clip): exactly one trailing newline
        // - `|+` (keep): preserve all trailing newlines
        let trailing_newlines = s.len() - s.trim_end_matches('\n').len();
        let content = s.trim_end_matches('\n');
        let lines: Vec<&str> = content.split('\n').collect();
        let first_line = lines.first().unwrap_or(&"");

        // Use indentation indicator (2) if first line begins with whitespace
        let needs_indent_indicator = first_line.starts_with(' ') || first_line.starts_with('\t');

        let indicator = match (needs_indent_indicator, trailing_newlines) {
            (true, 0) => "|2-",
            (true, 1) => "|2",
            (true, _) => "|2+",
            (false, 0) => "|-",
            (false, 1) => "|",
            (false, _) => "|+",
        };

        writeln!(w, "{}", indicator)?;

        let prefix = "  ".repeat(indent + 1);
        for line in &lines {
            writeln!(w, "{}{}", prefix, line)?;
        }

        // For |+ with multiple trailing newlines, add extra blank lines
        // (the content lines already contribute one newline each via writeln!,
        // and clip/keep modes add one more, so we need trailing_newlines - 1 extra)
        for _ in 1..trailing_newlines {
            writeln!(w)?;
        }
    } else if can_be_unquoted(s) {
        // Unquoted string
        writeln!(w, "{}", s)?;
    } else {
        // Double-quoted string
        writeln!(w, "{}", escape_string(s))?;
    }

    Ok(())
}

/// Write a YAML value at the given indentation level
/// `inline` indicates whether this is being written inline (after a key or
/// array marker)
fn write_yaml_value<W: Write>(w: &mut W, value: &Value, indent: usize, inline: bool) -> Result<()> {
    match value {
        Value::Null => {
            writeln!(w, "null")?;
        }
        Value::Bool(b) => {
            writeln!(w, "{}", if *b { "true" } else { "false" })?;
        }
        Value::Number(n) => {
            writeln!(w, "{}", n)?;
        }
        Value::String(s) => {
            write_yaml_string(w, s, indent)?;
        }
        Value::Array(arr) => {
            if arr.is_empty() {
                writeln!(w, "[]")?;
            } else {
                if inline {
                    writeln!(w)?;
                }
                let prefix = "  ".repeat(indent);
                for item in arr {
                    write!(w, "{}- ", prefix)?;
                    write_yaml_value(w, item, indent + 1, true)?;
                }
            }
        }
        Value::Object(obj) => {
            if obj.is_empty() {
                writeln!(w, "{{}}")?;
            } else {
                if inline {
                    writeln!(w)?;
                }
                let prefix = "  ".repeat(indent);
                for (key, val) in obj {
                    let formatted_key = if can_be_unquoted(key) {
                        key.clone()
                    } else {
                        escape_string(key)
                    };

                    write!(w, "{}{}:", prefix, formatted_key)?;

                    // Check if value needs to be on a new line
                    match val {
                        Value::Array(arr) if !arr.is_empty() => {
                            writeln!(w)?;
                            write_yaml_value(w, val, indent + 1, false)?;
                        }
                        Value::Object(obj) if !obj.is_empty() => {
                            writeln!(w)?;
                            write_yaml_value(w, val, indent + 1, false)?;
                        }
                        Value::String(s) if s.contains('\n') => {
                            write!(w, " ")?;
                            write_yaml_value(w, val, indent, true)?;
                        }
                        _ => {
                            write!(w, " ")?;
                            write_yaml_value(w, val, indent + 1, true)?;
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        serde_json::json,
    };

    /// Convert a JSON value to our YAML format and parse it back with
    /// serde_yaml
    fn round_trip(value: &Value) -> Value {
        let mut yaml_bytes = Vec::new();
        write_yaml_value(&mut yaml_bytes, value, 0, false).unwrap();
        let yaml_str = String::from_utf8(yaml_bytes).unwrap();
        serde_yaml::from_str(&yaml_str).unwrap()
    }

    #[test]
    fn test_scalars() {
        assert_eq!(round_trip(&json!(null)), json!(null));
        assert_eq!(round_trip(&json!(true)), json!(true));
        assert_eq!(round_trip(&json!(false)), json!(false));
        assert_eq!(round_trip(&json!(42)), json!(42));
        assert_eq!(round_trip(&json!(1.25)), json!(1.25));
        assert_eq!(round_trip(&json!("hello")), json!("hello"));
    }

    #[test]
    fn test_strings_needing_quotes() {
        assert_eq!(round_trip(&json!("123")), json!("123"));
        assert_eq!(round_trip(&json!("true")), json!("true"));
        assert_eq!(round_trip(&json!("null")), json!("null"));
        assert_eq!(round_trip(&json!("yes")), json!("yes"));
        assert_eq!(round_trip(&json!("")), json!(""));
        assert_eq!(round_trip(&json!("hello:world")), json!("hello:world"));
    }

    #[test]
    fn test_multiline_no_trailing() {
        assert_eq!(round_trip(&json!("a\nb")), json!("a\nb"));
        assert_eq!(
            round_trip(&json!("line1\nline2\nline3")),
            json!("line1\nline2\nline3")
        );
    }

    #[test]
    fn test_multiline_one_trailing() {
        assert_eq!(round_trip(&json!("a\nb\n")), json!("a\nb\n"));
        assert_eq!(round_trip(&json!("single\n")), json!("single\n"));
    }

    #[test]
    fn test_multiline_multiple_trailing() {
        assert_eq!(round_trip(&json!("a\nb\n\n")), json!("a\nb\n\n"));
        assert_eq!(round_trip(&json!("a\n\n\n")), json!("a\n\n\n"));
        assert_eq!(round_trip(&json!("x\n\n\n\n")), json!("x\n\n\n\n"));
    }

    #[test]
    fn test_multiline_leading_space() {
        assert_eq!(
            round_trip(&json!("  indented\nnormal")),
            json!("  indented\nnormal")
        );
        assert_eq!(
            round_trip(&json!("\ttabbed\nline")),
            json!("\ttabbed\nline")
        );
    }

    #[test]
    fn test_arrays() {
        assert_eq!(round_trip(&json!([])), json!([]));
        assert_eq!(round_trip(&json!([1, 2, 3])), json!([1, 2, 3]));
        assert_eq!(round_trip(&json!(["a", "b"])), json!(["a", "b"]));
        assert_eq!(
            round_trip(&json!([[1, 2], [3, 4]])),
            json!([[1, 2], [3, 4]])
        );
    }

    #[test]
    fn test_objects() {
        assert_eq!(round_trip(&json!({})), json!({}));
        assert_eq!(round_trip(&json!({"a": 1})), json!({"a": 1}));
        assert_eq!(
            round_trip(&json!({"nested": {"deep": true}})),
            json!({"nested": {"deep": true}})
        );
    }

    #[test]
    fn test_complex() {
        let complex = json!({
            "name": "test",
            "values": [1, 2, 3],
            "config": {
                "enabled": true,
                "script": "echo hello\necho world\n"
            }
        });
        assert_eq!(round_trip(&complex), complex);
    }
}
