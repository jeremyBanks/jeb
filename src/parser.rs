//! Annotation parsing
//! [impl _trace.syntax]

use crate::model::{Annotation, Location, Modifiers, SatisfactionMode};
use regex::Regex;
use std::path::PathBuf;
use std::sync::LazyLock;

/// Regex pattern for finding annotations
/// [impl _trace.syntax.brackets]
/// [impl _trace.syntax.not-preceded]
/// [impl _trace.syntax.not-followed]
static ANNOTATION_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    // Match [...] not preceded by ]/) and not followed by [/(
    // We handle the not-preceded/not-followed checks in code since lookbehind is limited
    Regex::new(r"\[([^\[\]]+)\]").unwrap()
});

/// Parse all annotations from file content
/// [impl _trace.syntax.structure]
/// [impl _trace.syntax.type]
pub fn parse_annotations(path: &PathBuf, content: &str, contexts: &[String]) -> Vec<Annotation> {
    let mut annotations = Vec::new();
    let lines: Vec<&str> = content.lines().collect();

    // Track whether we're inside a fenced code block
    let mut in_code_block = false;

    for (line_idx, line) in lines.iter().enumerate() {
        let line_num = line_idx + 1; // 1-indexed

        // Check for code fence markers (``` or ~~~)
        let trimmed = line.trim();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_block = !in_code_block;
            continue;
        }

        // Skip lines inside code blocks
        if in_code_block {
            continue;
        }

        for mat in ANNOTATION_PATTERN.find_iter(line) {
            let start = mat.start();
            let end = mat.end();

            // Check not-preceded constraint
            // [impl _trace.syntax.not-preceded]
            if start > 0 {
                let prev_char = line.chars().nth(start - 1);
                if prev_char == Some(']') || prev_char == Some(')') || prev_char == Some('`') {
                    continue;
                }
            }

            // Check not-followed constraint
            // [impl _trace.syntax.not-followed]
            if end < line.len() {
                let next_char = line.chars().nth(end);
                if next_char == Some('[') || next_char == Some('(') || next_char == Some('`') {
                    continue;
                }
            }

            // Extract inner content (without brackets)
            let inner = &line[start + 1..end - 1];

            // Parse the annotation
            if let Some(annotation) = parse_annotation_inner(
                inner,
                path.clone(),
                line_num,
                start + 1, // column is 1-indexed
                contexts.get(line_idx).cloned().unwrap_or_default(),
            ) {
                annotations.push(annotation);
            }
        }
    }

    annotations
}

/// Parse the inner content of an annotation
/// [impl _trace.syntax.structure]
fn parse_annotation_inner(
    inner: &str,
    file: PathBuf,
    line: usize,
    column: usize,
    context: String,
) -> Option<Annotation> {
    let parts: Vec<&str> = inner.split_whitespace().collect();

    // Must have at least 2 components: type and ID
    // [impl _trace.syntax.structure]
    if parts.len() < 2 {
        return None;
    }

    // [impl _trace.types.custom]
    let kind = parts[0].to_string();

    // Find the ID - it's the last component that looks like an ID
    // (not a modifier like @child or +type or -type)
    // The ID must be after the type, so we look backwards from the end
    let mut id_idx = parts.len() - 1;
    while id_idx > 0 {
        let part = parts[id_idx];
        // Skip modifiers (start with @ or + or -)
        if part.starts_with('@') || part.starts_with('+') || part.starts_with('-') {
            id_idx -= 1;
        } else {
            break;
        }
    }

    // If we went all the way back to the type, there's no valid ID
    if id_idx == 0 {
        return None;
    }

    let id = parts[id_idx].to_string();

    // Validate the ID
    // [impl _trace.syntax.id]
    // [impl _trace.syntax.id.segments]
    // [impl _trace.syntax.id.minimum]
    if !is_valid_id(&id) {
        return None;
    }

    // Parse modifiers from components after the ID (for def annotations)
    let modifiers = if kind == "def" {
        parse_modifiers(&parts[id_idx + 1..])
    } else {
        Modifiers::default()
    };

    Some(Annotation {
        kind,
        id,
        modifiers,
        location: Location::new(file, line, column),
        context,
    })
}

/// Validate a requirement ID
/// [impl _trace.syntax.id]
/// [impl _trace.syntax.id.segments]
/// [impl _trace.syntax.id.minimum]
pub fn is_valid_id(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }
    // Must not start or end with a period
    if s.starts_with('.') || s.ends_with('.') {
        return false;
    }
    // Must not have consecutive periods
    if s.contains("..") {
        return false;
    }

    // Each segment must be non-empty and contain only alphanumeric, dash, underscore
    s.split('.').all(|seg| {
        !seg.is_empty()
            && seg
                .chars()
                .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
    })
}

/// Parse modifiers from annotation components
/// [impl _trace.satisfaction.modifiers]
fn parse_modifiers(parts: &[&str]) -> Modifiers {
    let mut modifiers = Modifiers::default();

    for part in parts {
        if part.starts_with('@') {
            // Satisfaction mode
            match *part {
                "@self" => modifiers.mode = Some(SatisfactionMode::Self_),
                "@child" => modifiers.mode = Some(SatisfactionMode::Child),
                "@either" => modifiers.mode = Some(SatisfactionMode::Either),
                _ => {}
            }
        } else if part.starts_with('+') || part.starts_with('-') {
            // Type modifiers like +doc-test or +impl+test
            parse_type_modifiers(part, &mut modifiers);
        }
    }

    modifiers
}

/// Parse type modifiers like +doc-test or +impl+test
fn parse_type_modifiers(s: &str, modifiers: &mut Modifiers) {
    let mut current_add = true;
    let mut current_type = String::new();

    for c in s.chars() {
        match c {
            '+' => {
                if !current_type.is_empty() {
                    if current_add {
                        modifiers.add_types.push(current_type.clone());
                    } else {
                        modifiers.remove_types.push(current_type.clone());
                    }
                    current_type.clear();
                }
                current_add = true;
            }
            '-' => {
                if !current_type.is_empty() {
                    if current_add {
                        modifiers.add_types.push(current_type.clone());
                    } else {
                        modifiers.remove_types.push(current_type.clone());
                    }
                    current_type.clear();
                }
                current_add = false;
            }
            _ => {
                current_type.push(c);
            }
        }
    }

    // Don't forget the last type
    if !current_type.is_empty() {
        if current_add {
            modifiers.add_types.push(current_type);
        } else {
            modifiers.remove_types.push(current_type);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [test _trace.syntax.id]
    /// [test _trace.syntax.id.segments]
    #[test]
    fn test_valid_ids() {
        assert!(is_valid_id("foo"));
        assert!(is_valid_id("foo.bar"));
        assert!(is_valid_id("foo.bar.baz"));
        assert!(is_valid_id("foo-bar"));
        assert!(is_valid_id("foo_bar"));
        assert!(is_valid_id("foo.bar-baz_qux"));
        assert!(is_valid_id("_trace"));
        assert!(is_valid_id("_trace.files"));
        assert!(is_valid_id("_trace.syntax.id"));
    }

    /// [test _trace.syntax.id]
    /// [test _trace.syntax.id.minimum]
    #[test]
    fn test_invalid_ids() {
        assert!(!is_valid_id(""));
        assert!(!is_valid_id(".foo"));
        assert!(!is_valid_id("foo."));
        assert!(!is_valid_id("foo..bar"));
        assert!(!is_valid_id("foo.bar!"));
        assert!(!is_valid_id("foo bar"));
    }

    /// [test _trace.satisfaction.modifiers]
    #[test]
    fn test_parse_modifiers() {
        let mods = parse_modifiers(&["+doc", "-test"]);
        assert_eq!(mods.add_types, vec!["doc"]);
        assert_eq!(mods.remove_types, vec!["test"]);

        let mods2 = parse_modifiers(&["+doc-test"]);
        assert_eq!(mods2.add_types, vec!["doc"]);
        assert_eq!(mods2.remove_types, vec!["test"]);

        let mods3 = parse_modifiers(&["@child"]);
        assert_eq!(mods3.mode, Some(SatisfactionMode::Child));
    }

    /// [test _trace.syntax.brackets]
    #[test]
    fn test_brackets_detection() {
        let path = PathBuf::from("test.md");
        let content = "Here is [impl some.id] in text";
        let contexts = vec![String::new()];
        let annotations = parse_annotations(&path, content, &contexts);
        assert_eq!(annotations.len(), 1);
        assert_eq!(annotations[0].kind, "impl");
        assert_eq!(annotations[0].id, "some.id");
    }

    /// [test _trace.syntax.not-preceded]
    #[test]
    fn test_not_preceded() {
        let path = PathBuf::from("test.md");
        // Markdown link syntax should NOT be parsed
        let content = "[text](url) and [text][ref]";
        let contexts = vec![String::new()];
        let annotations = parse_annotations(&path, content, &contexts);
        assert!(annotations.is_empty(), "Markdown links should be ignored");
    }

    /// [test _trace.syntax.not-followed]
    #[test]
    fn test_not_followed() {
        let path = PathBuf::from("test.md");
        // Text followed by [ or ( should be ignored
        let content = "[link](url) and [ref][target]";
        let contexts = vec![String::new()];
        let annotations = parse_annotations(&path, content, &contexts);
        assert!(annotations.is_empty(), "Text followed by [ or ( should be ignored");
    }

    /// [test _trace.syntax.structure]
    /// [test _trace.syntax.type]
    #[test]
    fn test_structure_requires_two_parts() {
        let path = PathBuf::from("test.md");
        // Single word should NOT be an annotation
        let content = "[single] but [impl valid.id] works";
        let contexts = vec![String::new()];
        let annotations = parse_annotations(&path, content, &contexts);
        assert_eq!(annotations.len(), 1, "Only [impl valid.id] should be found");
        assert_eq!(annotations[0].kind, "impl");
    }
}
