use {
    anyhow::{Context, Result, bail},
    serde_json::Value,
    std::io::{self, BufRead, Write},
};

fn main() -> Result<()> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

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
        let lines: Vec<&str> = s.split('\n').collect();
        let first_line = lines.first().unwrap_or(&"");

        // Use |2- if first line begins with whitespace
        let indicator = if first_line.starts_with(' ') || first_line.starts_with('\t') {
            "|2-"
        } else {
            "|-"
        };

        writeln!(w, "{}", indicator)?;

        let prefix = "  ".repeat(indent + 1);
        for line in &lines {
            writeln!(w, "{}{}", prefix, line)?;
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
/// `inline` indicates whether this is being written inline (after a key or array marker)
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
