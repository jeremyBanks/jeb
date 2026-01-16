//! A proc-macro that stringifies tokens while preserving original whitespace.
//!
//! Unlike the built-in `stringify!` macro which normalizes whitespace to single
//! spaces, `stringify_verbatim!` uses span information to reconstruct the
//! original formatting.
//!
//! # Example
//!
//! ```
//! use stringify_verbatim::stringify_verbatim;
//!
//! // Built-in stringify normalizes whitespace:
//! // stringify!(foo   bar) → "foo bar"
//!
//! // stringify_verbatim preserves it:
//! let s = stringify_verbatim!(foo   bar);
//! assert_eq!(s, "foo   bar");
//! ```
//!
//! # Limitations
//!
//! - Comments are not preserved (they're not part of the token stream)
//! - Trailing whitespace after the last token cannot be captured
//! - Accuracy depends on `proc_macro2`'s span-locations feature

use {
    proc_macro::TokenStream,
    proc_macro2::{
        LineColumn,
        TokenTree,
    },
};

/// Stringify tokens while preserving original whitespace.
///
/// Returns a `&'static str` containing the tokens with their original spacing.
///
/// # Example
///
/// ```
/// use stringify_verbatim::stringify_verbatim;
///
/// let code = stringify_verbatim!(
///     fn example() {
///         42
///     }
/// );
/// assert!(code.contains("\n")); // Newlines preserved
/// ```
#[proc_macro]
pub fn stringify_verbatim(input: TokenStream) -> TokenStream {
    let input2: proc_macro2::TokenStream = input.into();

    // Debug: print token spans
    if std::env::var("DEBUG_STRINGIFY_VERBATIM").is_ok() {
        eprintln!("=== stringify_verbatim input ===");
        debug_print_spans(&input2.clone().into_iter().collect::<Vec<_>>(), 0);
        eprintln!("================================");
    }

    let result = reconstruct_with_whitespace(input2.clone());

    if std::env::var("DEBUG_STRINGIFY_VERBATIM").is_ok() {
        let tts: Vec<TokenTree> = input2.into_iter().collect();
        let bounds = find_valid_bounds(&tts);
        eprintln!("Bounds: min={} max={}", bounds.min_line, bounds.max_line);
        eprintln!("Result: {:?}", result);
    }

    // Return as a string literal
    let lit = proc_macro2::Literal::string(&result);
    proc_macro2::TokenStream::from(proc_macro2::TokenTree::Literal(lit)).into()
}

fn debug_print_spans(tts: &[TokenTree], indent: usize) {
    for tt in tts {
        let span = tt.span();
        let start = span.start();
        let prefix = " ".repeat(indent);
        match tt {
            TokenTree::Group(g) => {
                eprintln!("{}Group {:?} @ {}:{}", prefix, g.delimiter(), start.line, start.column);
                let inner: Vec<TokenTree> = g.stream().into_iter().collect();
                debug_print_spans(&inner, indent + 2);
            }
            TokenTree::Ident(i) => {
                eprintln!("{}Ident '{}' @ {}:{}", prefix, i, start.line, start.column);
            }
            TokenTree::Punct(p) => {
                eprintln!("{}Punct '{}' @ {}:{}", prefix, p, start.line, start.column);
            }
            TokenTree::Literal(l) => {
                eprintln!("{}Literal {} @ {}:{}", prefix, l, start.line, start.column);
            }
        }
    }
}

/// Represents the valid line range for "correct" token positions.
/// Tokens outside this range are considered to have wrong positions from macro
/// expansion.
struct ValidBounds {
    min_line: usize,
    max_line: usize,
}

impl ValidBounds {
    /// Check if a line number is within the valid bounds
    fn contains(&self, line: usize) -> bool {
        line >= self.min_line && line <= self.max_line
    }
}

/// Find valid bounds by identifying the densest cluster of line numbers.
/// The idea: correct tokens cluster together in a relatively narrow line range,
/// while wrong tokens (from macro expansion) scatter to unrelated line numbers.
fn find_valid_bounds(tts: &[TokenTree]) -> ValidBounds {
    if tts.is_empty() {
        return ValidBounds {
            min_line: 0,
            max_line: usize::MAX,
        };
    }

    // Collect all line numbers from all tokens (flattening groups)
    let mut all_lines: Vec<usize> = Vec::new();
    fn collect_lines(tt: &TokenTree, lines: &mut Vec<usize>) {
        lines.push(tt.span().start().line);
        if let TokenTree::Group(g) = tt {
            for inner in g.stream() {
                collect_lines(&inner, lines);
            }
        }
    }
    for tt in tts {
        collect_lines(tt, &mut all_lines);
    }

    if all_lines.is_empty() {
        return ValidBounds {
            min_line: 0,
            max_line: usize::MAX,
        };
    }

    // Group lines into buckets (regions of 20 lines) and count density
    use std::collections::HashMap;
    let mut buckets: HashMap<usize, usize> = HashMap::new();
    for &line in &all_lines {
        *buckets.entry(line / 20).or_default() += 1;
    }

    // Find the densest bucket (most tokens in a 20-line region)
    let (&densest_bucket, &max_count) = buckets
        .iter()
        .max_by_key(|(_, &count)| count)
        .unwrap_or((&0, &0));

    // Check if there's a clear winner or if it's ambiguous
    let total_tokens = all_lines.len();
    let density_threshold = total_tokens / 3; // At least 1/3 of tokens should be in the main cluster

    if max_count < density_threshold && buckets.len() > 1 {
        // No clear cluster - tokens are scattered
        // Fall back to accepting a wide range around the densest area
        let center_line = densest_bucket * 20 + 10;
        return ValidBounds {
            min_line: center_line.saturating_sub(50),
            max_line: center_line.saturating_add(50),
        };
    }

    // Expand from the densest bucket to include adjacent populated buckets
    let mut min_bucket = densest_bucket;
    let mut max_bucket = densest_bucket;

    // Expand down while adjacent buckets have reasonable token counts
    while min_bucket > 0 {
        let prev_bucket = min_bucket - 1;
        if let Some(&count) = buckets.get(&prev_bucket) {
            if count >= max_count / 4 {
                min_bucket = prev_bucket;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // Expand up while adjacent buckets have reasonable token counts
    loop {
        let next_bucket = max_bucket + 1;
        if let Some(&count) = buckets.get(&next_bucket) {
            if count >= max_count / 4 {
                max_bucket = next_bucket;
            } else {
                break;
            }
        } else {
            break;
        }
    }

    // Convert bucket range to line range with margin
    ValidBounds {
        min_line: (min_bucket * 20).saturating_sub(5),
        max_line: (max_bucket + 1) * 20 + 10,
    }
}

/// Reconstruct the token stream as a string, preserving whitespace from span
/// info.
fn reconstruct_with_whitespace(tokens: proc_macro2::TokenStream) -> String {
    let tts: Vec<TokenTree> = tokens.into_iter().collect();

    if tts.is_empty() {
        return String::new();
    }

    // Find valid bounds by walking from both ends
    let bounds = find_valid_bounds(&tts);

    // Find the baseline (starting position) from tokens within bounds
    let baseline = find_first_span_start_bounded(&tts, &bounds);

    let mut result = String::new();
    let mut prev_end: Option<LineColumn> = None;
    let mut i = 0;

    while i < tts.len() {
        // Try to detect and convert doc attributes back to /// or //! syntax
        if let Some((doc_comment, consumed, end_pos)) = try_parse_doc_attribute(&tts[i..]) {
            // Add whitespace before the doc comment
            if let Some(prev) = prev_end {
                let start = tts[i].span().start();
                let ws = compute_whitespace_bounded(prev, start, baseline, &bounds);
                result.push_str(&ws);
            }

            result.push_str(&doc_comment);
            prev_end = Some(end_pos);
            i += consumed;
            continue;
        }

        let tt = &tts[i];
        let span = tt.span();
        let start = span.start();
        let end = span.end();

        // Add whitespace between previous token and this one
        if let Some(prev) = prev_end {
            let ws = compute_whitespace_bounded(prev, start, baseline, &bounds);
            result.push_str(&ws);
        }

        // Add the token's text
        result.push_str(&token_to_string_bounded(tt, baseline, &bounds));

        prev_end = Some(end);
        i += 1;
    }

    result
}

/// Find the first span start position, but only considering tokens within
/// bounds.
fn find_first_span_start_bounded(tts: &[TokenTree], bounds: &ValidBounds) -> LineColumn {
    if tts.is_empty() {
        return LineColumn { line: 1, column: 0 };
    }

    let mut min_line = usize::MAX;
    let mut min_column = 0;

    fn find_min_in_tree_bounded(
        tt: &TokenTree,
        min_line: &mut usize,
        min_column: &mut usize,
        bounds: &ValidBounds,
    ) {
        let start = tt.span().start();
        // Only consider tokens within bounds
        if bounds.contains(start.line) {
            if start.line < *min_line || (start.line == *min_line && start.column < *min_column) {
                *min_line = start.line;
                *min_column = start.column;
            }
        }
        if let TokenTree::Group(g) = tt {
            for inner in g.stream() {
                find_min_in_tree_bounded(&inner, min_line, min_column, bounds);
            }
        }
    }

    for tt in tts {
        find_min_in_tree_bounded(tt, &mut min_line, &mut min_column, bounds);
    }

    if min_line == usize::MAX {
        // No tokens within bounds, fall back to first token
        tts[0].span().start()
    } else {
        LineColumn {
            line: min_line,
            column: min_column,
        }
    }
}

/// Compute whitespace, using bounds to detect invalid positions.
fn compute_whitespace_bounded(
    from: LineColumn,
    to: LineColumn,
    baseline: LineColumn,
    bounds: &ValidBounds,
) -> String {
    let from_valid = bounds.contains(from.line);
    let to_valid = bounds.contains(to.line);

    // If either position is out of bounds, use simple spacing
    if !from_valid || !to_valid {
        return " ".to_string();
    }

    // Both positions are valid - use normal whitespace computation
    compute_whitespace(from, to, baseline)
}

/// Convert a token tree to string, using bounds to handle invalid positions.
fn token_to_string_bounded(tt: &TokenTree, baseline: LineColumn, bounds: &ValidBounds) -> String {
    match tt {
        TokenTree::Group(g) => {
            let inner_tokens: Vec<TokenTree> = g.stream().into_iter().collect();
            let (open, close) = match g.delimiter() {
                proc_macro2::Delimiter::Parenthesis => ("(", ")"),
                proc_macro2::Delimiter::Brace => ("{", "}"),
                proc_macro2::Delimiter::Bracket => ("[", "]"),
                proc_macro2::Delimiter::None => ("", ""),
            };

            if inner_tokens.is_empty() {
                return format!("{}{}", open, close);
            }

            let group_span = g.span();
            let first_token = inner_tokens.first().unwrap();
            let last_token = inner_tokens.last().unwrap();

            let group_start = group_span.start();
            let first_start = first_token.span().start();

            // Check if group position is valid
            let group_valid = bounds.contains(group_start.line);
            let first_valid = bounds.contains(first_start.line);

            let leading_ws = if !group_valid || !first_valid {
                // One or both positions invalid - use minimal spacing
                if g.delimiter() == proc_macro2::Delimiter::Brace {
                    "\n    ".to_string() // Indent block contents
                } else {
                    String::new()
                }
            } else if first_start.line == group_start.line {
                let diff = first_start.column.saturating_sub(group_start.column);
                if diff > 1 && diff <= 20 {
                    " ".repeat(diff - 1)
                } else {
                    String::new()
                }
            } else {
                let line_diff = first_start.line.saturating_sub(group_start.line);
                if line_diff > 5 {
                    "\n".to_string()
                } else {
                    let mut ws = "\n".repeat(line_diff);
                    let indent = first_start.column.saturating_sub(baseline.column);
                    ws.push_str(&" ".repeat(indent));
                    ws
                }
            };

            let group_end = group_span.end();
            let last_end = last_token.span().end();
            let group_end_valid = bounds.contains(group_end.line);
            let last_valid = bounds.contains(last_end.line);

            let trailing_ws = if !group_end_valid || !last_valid {
                if g.delimiter() == proc_macro2::Delimiter::Brace {
                    "\n".to_string()
                } else {
                    String::new()
                }
            } else if last_end.line == group_end.line {
                let diff = group_end.column.saturating_sub(last_end.column);
                if diff > 1 && diff <= 20 {
                    " ".repeat(diff - 1)
                } else {
                    String::new()
                }
            } else {
                let line_diff = group_end.line.saturating_sub(last_end.line);
                if line_diff > 5 {
                    String::new()
                } else {
                    let mut ws = "\n".repeat(line_diff);
                    let normalized_column = group_end.column.saturating_sub(baseline.column);
                    if normalized_column > 0 {
                        ws.push_str(&" ".repeat(normalized_column.saturating_sub(1)));
                    }
                    ws
                }
            };

            // Reconstruct inner content
            let inner = reconstruct_with_whitespace_bounded(g.stream(), baseline, bounds);

            format!("{}{}{}{}{}", open, leading_ws, inner, trailing_ws, close)
        }
        TokenTree::Ident(i) => i.to_string(),
        TokenTree::Punct(p) => p.to_string(),
        TokenTree::Literal(l) => l.to_string(),
    }
}

/// Reconstruct with bounds checking.
fn reconstruct_with_whitespace_bounded(
    tokens: proc_macro2::TokenStream,
    baseline: LineColumn,
    bounds: &ValidBounds,
) -> String {
    let tts: Vec<TokenTree> = tokens.into_iter().collect();

    if tts.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    let mut prev_end: Option<LineColumn> = None;

    for tt in &tts {
        let span = tt.span();
        let start = span.start();
        let end = span.end();

        if let Some(prev) = prev_end {
            let ws = compute_whitespace_bounded(prev, start, baseline, bounds);
            result.push_str(&ws);
        }

        result.push_str(&token_to_string_bounded(tt, baseline, bounds));
        prev_end = Some(end);
    }

    result
}

/// Try to parse a doc attribute pattern and convert it back to /// or //!
/// syntax. Returns (doc_comment_string, tokens_consumed, end_position) if
/// successful.
///
/// Only converts if the span positions indicate this was originally a `///`
/// comment (where all tokens map to the same location), NOT an explicit `#[doc
/// = "..."]`.
fn try_parse_doc_attribute(tokens: &[TokenTree]) -> Option<(String, usize, LineColumn)> {
    // Pattern: # [ doc = "..." ] or # ! [ doc = "..." ]
    if tokens.is_empty() {
        return None;
    }

    // Check for #
    let hash_span = match &tokens[0] {
        TokenTree::Punct(p) if p.as_char() == '#' => p.span(),
        _ => return None,
    };

    let mut idx = 1;
    let is_inner;

    // Check for optional !
    if idx < tokens.len() {
        if let TokenTree::Punct(p) = &tokens[idx] {
            if p.as_char() == '!' {
                is_inner = true;
                idx += 1;
            } else {
                is_inner = false;
            }
        } else {
            is_inner = false;
        }
    } else {
        return None;
    }

    // Check for [...]
    if idx >= tokens.len() {
        return None;
    }

    let group = match &tokens[idx] {
        TokenTree::Group(g) if g.delimiter() == proc_macro2::Delimiter::Bracket => g,
        _ => return None,
    };

    // Parse the group contents: doc = "..." or doc="..."
    let inner: Vec<TokenTree> = group.stream().into_iter().collect();

    // Need at least: doc = "string" (3 tokens) or doc="string" with no space (still
    // 3)
    if inner.len() < 3 {
        return None;
    }

    // Check for "doc" ident
    let is_doc = match &inner[0] {
        TokenTree::Ident(i) => *i == "doc",
        _ => false,
    };

    if !is_doc {
        return None;
    }

    // Check for =
    let has_eq = match &inner[1] {
        TokenTree::Punct(p) => p.as_char() == '=',
        _ => false,
    };

    if !has_eq {
        return None;
    }

    // Check for string literal and properly parse it to handle escape sequences
    let doc_content = match &inner[2] {
        TokenTree::Literal(lit) => {
            // Use syn to properly parse the string literal and unescape it
            let token_stream: proc_macro2::TokenStream = TokenTree::Literal(lit.clone()).into();
            let lit_str: syn::LitStr = syn::parse2(token_stream).ok()?;
            lit_str.value()
        }
        _ => return None,
    };

    // Check if this was originally a /// comment or an explicit #[doc = "..."]
    // For /// comments, the # and [ tokens are synthetic and have identical spans
    // (both point to the original /// location).
    // For explicit #[doc = "..."], the [ starts one column after the #.
    let hash_start = hash_span.start();
    let group_start = group.span().start();

    // For "/// comment" (synthetic):
    //   - Both hash and group have the same start position (line AND column)
    // For "#[doc = "comment"]" (explicit):
    //   - hash is at position of #, group starts at position of [
    let is_synthetic =
        hash_start.line == group_start.line && hash_start.column == group_start.column;

    if !is_synthetic {
        // This is an explicit #[doc = "..."], don't convert
        return None;
    }

    // Successfully parsed a synthetic doc attribute from ///!
    let prefix = if is_inner { "//!" } else { "///" };
    let doc_comment = format!("{}{}", prefix, doc_content);

    let end_pos = group.span().end();
    let consumed = idx + 1;

    Some((doc_comment, consumed, end_pos))
}

/// Compute the whitespace string between two positions, using baseline for
/// column normalization on newlines.
///
/// If span positions appear discontinuous (which happens with macro-expanded
/// tokens), use minimal spacing to avoid creating huge gaps.
fn compute_whitespace(from: LineColumn, to: LineColumn, baseline: LineColumn) -> String {
    // Check if the span positions look suspicious (macro expansion artifacts)
    // A common sign is: different lines but to.line < from.line (going backwards)
    // or same line but to.column < from.column (going backwards on same line)
    let going_backwards = to.line < from.line || (to.line == from.line && to.column < from.column);

    if going_backwards {
        // Spans are clearly wrong - just use a single space
        return " ".to_string();
    }

    if from.line == to.line {
        // Same line: just spaces
        let spaces = to.column.saturating_sub(from.column);
        // If there's a big gap on the same line, it might be due to macro expansion
        // Just use a single space in that case
        if spaces > 20 {
            " ".to_string()
        } else if spaces == 0 {
            // No space needed (adjacent tokens)
            String::new()
        } else {
            " ".repeat(spaces)
        }
    } else {
        // Different lines: check if this looks like normal sequential code
        // or discontinuous macro-expanded tokens
        let line_diff = to.line.saturating_sub(from.line);

        // If the line difference is too large (> 5 lines), it's probably
        // macro expansion artifacts - use just a single newline
        if line_diff > 5 {
            "\n".to_string()
        } else {
            // Normal case: newlines + indentation relative to baseline
            let mut ws = "\n".repeat(line_diff);
            // Normalize column relative to baseline
            let normalized_column = to.column.saturating_sub(baseline.column);
            ws.push_str(&" ".repeat(normalized_column));
            ws
        }
    }
}

#[cfg(test)]
mod tests {
    // Tests need to be in a separate crate that uses this proc-macro
    // See tests/ directory
}
