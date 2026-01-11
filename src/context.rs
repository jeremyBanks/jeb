//! Context extraction from source files
//! [impl _trace.context]

/// Check if a line has any alphanumeric characters
/// [impl _trace.context.boundaries]
fn has_alphanumeric(line: &str) -> bool {
    line.chars().any(|c| c.is_alphanumeric())
}

/// Extract context for each line in a file
/// [impl _trace.context.boundaries]
/// [impl _trace.context.shared]
pub fn extract_contexts(content: &str) -> Vec<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut contexts = Vec::with_capacity(lines.len());

    for line_idx in 0..lines.len() {
        let context = extract_context_for_line(&lines, line_idx);
        contexts.push(context);
    }

    contexts
}

/// Extract context for a specific line
/// [impl _trace.context.boundaries]
fn extract_context_for_line(lines: &[&str], line_idx: usize) -> String {
    // Find the start of the context block (scan upward)
    let mut start = line_idx;
    while start > 0 {
        if !has_alphanumeric(lines[start - 1]) {
            break;
        }
        start -= 1;
    }

    // Find the end of the context block (scan downward)
    let mut end = line_idx;
    while end < lines.len() - 1 {
        if !has_alphanumeric(lines[end + 1]) {
            break;
        }
        end += 1;
    }

    // Collect the context lines
    let context_lines: Vec<&str> = lines[start..=end].to_vec();

    // Strip common prefix
    let stripped = strip_common_prefix(&context_lines);

    stripped.join("\n")
}

/// Strip common non-alphanumeric prefix from all lines
/// [impl _trace.context.prefix-strip]
fn strip_common_prefix(lines: &[&str]) -> Vec<String> {
    if lines.is_empty() {
        return vec![];
    }

    // Find the common prefix length from the first line
    // The prefix consists of non-alphanumeric chars (excluding dash, underscore, period)
    let first_line = lines[0];
    let prefix_len = first_line
        .chars()
        .take_while(|c| !c.is_alphanumeric() && *c != '-' && *c != '_' && *c != '.')
        .count();

    if prefix_len == 0 {
        return lines.iter().map(|l| l.to_string()).collect();
    }

    // Get the actual prefix string (handling multi-byte chars properly)
    let prefix: String = first_line.chars().take(prefix_len).collect();

    // Check if all lines start with this prefix
    let all_have_prefix = lines.iter().all(|l| l.starts_with(&prefix));

    if all_have_prefix {
        lines
            .iter()
            .map(|l| l.chars().skip(prefix_len).collect())
            .collect()
    } else {
        lines.iter().map(|l| l.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [test _trace.context.boundaries]
    #[test]
    fn test_has_alphanumeric() {
        assert!(has_alphanumeric("hello"));
        assert!(has_alphanumeric("  hello  "));
        assert!(has_alphanumeric("123"));
        assert!(!has_alphanumeric(""));
        assert!(!has_alphanumeric("---"));
        assert!(!has_alphanumeric("   "));
    }

    /// [test _trace.context.prefix-strip]
    #[test]
    fn test_strip_common_prefix() {
        let lines = vec!["// hello", "// world", "// test"];
        let stripped = strip_common_prefix(&lines);
        assert_eq!(stripped, vec!["hello", "world", "test"]);
    }

    /// [test _trace.context.prefix-strip]
    #[test]
    fn test_strip_common_prefix_no_common() {
        let lines = vec!["// hello", "world", "// test"];
        let stripped = strip_common_prefix(&lines);
        assert_eq!(stripped, vec!["// hello", "world", "// test"]);
    }

    /// [test _trace.context]
    /// [test _trace.context.boundaries]
    /// [test _trace.context.shared]
    #[test]
    fn test_context_extraction() {
        let content = "// First line\n// Second line\n// Third line\n\n// Another block";
        let contexts = extract_contexts(content);

        // Lines 0, 1, 2 should share the same context (no blank line between them)
        assert_eq!(contexts[0], contexts[1]);
        assert_eq!(contexts[1], contexts[2]);

        // Line 4 should have different context (after blank line)
        assert_ne!(contexts[2], contexts[4]);
    }
}
