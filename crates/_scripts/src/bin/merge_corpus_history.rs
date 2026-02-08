use {
    _chosen::{
        bytes_to_text,
        text_to_bytes,
    },
    anyhow::{
        Context,
        Result,
    },
    std::{
        collections::BTreeSet,
        process::Command,
    },
};

/// Represents a corpus entry with type and raw data
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Entry {
    entry_type: String,
    data: Vec<u8>,
}

impl Entry {
    /// Parse a line - handles both current format (type:encoded) and old JSON
    /// format
    fn parse(line: &str) -> Option<Self> {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return None;
        }

        // Try JSON format first: {"type":"corpus","data":"..."}
        if line.starts_with('{')
            && let Some(entry) = Self::parse_json(line)
        {
            return Some(entry);
        }

        // Current format: type:encoded_data
        let (entry_type, encoded) = line.split_once(':')?;
        if entry_type.is_empty()
            || !entry_type
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '_')
        {
            return None;
        }

        let data = text_to_bytes(encoded)?;
        Some(Entry {
            entry_type: entry_type.to_string(),
            data,
        })
    }

    /// Parse old JSON format: {"type":"corpus","data":"..."}
    fn parse_json(line: &str) -> Option<Self> {
        // Simple JSON parsing without pulling in serde_json
        let line = line.trim();
        if !line.starts_with('{') || !line.ends_with('}') {
            return None;
        }

        let inner = &line[1..line.len() - 1];

        // Find "type":"..." and "data":"..."
        let entry_type = extract_json_string(inner, "type")?;
        let data_str = extract_json_string(inner, "data")?;

        // The data field in old format used JSON string escaping, not our encoding
        let data = unescape_json_string(&data_str)?;

        Some(Entry { entry_type, data })
    }

    /// Format as current format: type:encoded_data
    fn to_line(&self) -> String {
        format!("{}:{}", self.entry_type, bytes_to_text(&self.data))
    }
}

/// Extract a string value from JSON-like content for a given key
fn extract_json_string(content: &str, key: &str) -> Option<String> {
    let pattern = format!("\"{}\":\"", key);
    let start = content.find(&pattern)? + pattern.len();
    let rest = &content[start..];

    // Find the closing quote, handling escapes
    let mut result = String::new();
    let mut chars = rest.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '"' {
            return Some(result);
        } else if c == '\\' {
            // Just include the escape sequence as-is for now
            result.push(c);
            if chars.peek().is_some() {
                result.push(chars.next().unwrap());
            }
        } else {
            result.push(c);
        }
    }
    None
}

/// Unescape a JSON string (handles \n, \t, \r, \\, \", \uXXXX, etc.)
fn unescape_json_string(s: &str) -> Option<Vec<u8>> {
    let mut result = Vec::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next()? {
                'n' => result.push(b'\n'),
                'r' => result.push(b'\r'),
                't' => result.push(b'\t'),
                '\\' => result.push(b'\\'),
                '"' => result.push(b'"'),
                '/' => result.push(b'/'),
                'b' => result.push(0x08),
                'f' => result.push(0x0C),
                'u' => {
                    // \uXXXX
                    let hex: String = chars.by_ref().take(4).collect();
                    if hex.len() != 4 {
                        return None;
                    }
                    let code = u16::from_str_radix(&hex, 16).ok()?;
                    // For simplicity, just encode as UTF-8 if it's valid
                    if let Some(ch) = char::from_u32(code as u32) {
                        let mut buf = [0u8; 4];
                        let encoded = ch.encode_utf8(&mut buf);
                        result.extend_from_slice(encoded.as_bytes());
                    } else {
                        // Invalid unicode, just skip or encode as replacement
                        result.extend_from_slice("\u{FFFD}".as_bytes());
                    }
                }
                'x' => {
                    // \xXX (non-standard but sometimes used)
                    let hex: String = chars.by_ref().take(2).collect();
                    if hex.len() != 2 {
                        return None;
                    }
                    let byte = u8::from_str_radix(&hex, 16).ok()?;
                    result.push(byte);
                }
                other => {
                    // Unknown escape, just include literally
                    result.push(b'\\');
                    let mut buf = [0u8; 4];
                    let encoded = other.encode_utf8(&mut buf);
                    result.extend_from_slice(encoded.as_bytes());
                }
            }
        } else {
            let mut buf = [0u8; 4];
            let encoded = c.encode_utf8(&mut buf);
            result.extend_from_slice(encoded.as_bytes());
        }
    }

    Some(result)
}

fn main() -> Result<()> {
    let file_path = "crates/_examples/fuzz/fuzz_targets/fuzz_json.corpus";

    // Get all commits that touched this file
    let output = Command::new("git")
        .args(["log", "--pretty=format:%H", "--follow", "--", file_path])
        .output()
        .context("failed to run git log")?;

    if !output.status.success() {
        anyhow::bail!(
            "git log failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let commits: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    eprintln!("Found {} commits that touched {}", commits.len(), file_path);

    let mut all_entries: BTreeSet<Entry> = BTreeSet::new();

    // Also read the current working tree version
    if let Ok(content) = std::fs::read_to_string(file_path) {
        for line in content.lines() {
            if let Some(entry) = Entry::parse(line) {
                all_entries.insert(entry);
            }
        }
        eprintln!("Read {} entries from working tree", all_entries.len());
    }

    // Read the file content at each commit
    for commit in &commits {
        let output = Command::new("git")
            .args(["show", &format!("{}:{}", commit, file_path)])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let content = String::from_utf8_lossy(&output.stdout);
                let before = all_entries.len();
                for line in content.lines() {
                    if let Some(entry) = Entry::parse(line) {
                        all_entries.insert(entry);
                    }
                }
                let added = all_entries.len() - before;
                if added > 0 {
                    eprintln!("Commit {}: +{} new entries", &commit[..8], added);
                }
            }
            _ => {
                // File might not exist at this commit (renamed, etc.)
                eprintln!("Commit {}: file not found (possibly renamed)", &commit[..8]);
            }
        }
    }

    eprintln!("\nTotal unique entries: {}", all_entries.len());

    // Output all entries in current format, sorted
    for entry in &all_entries {
        println!("{}", entry.to_line());
    }

    Ok(())
}
