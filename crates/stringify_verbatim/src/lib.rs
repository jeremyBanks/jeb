//! A proc-macro that stringifies tokens while preserving original whitespace.
//!
//! Unlike the built-in `stringify!` macro which normalizes whitespace to single
//! spaces, `stringify_verbatim!` uses span information to reconstruct the
//! original formatting.

use proc_macro::TokenStream;
use proc_macro2::{LineColumn, TokenTree};

#[proc_macro]
pub fn stringify_verbatim(input: TokenStream) -> TokenStream {
    let input2: proc_macro2::TokenStream = input.into();
    let tts: Vec<TokenTree> = input2.into_iter().collect();
    let result = reconstruct(&tts);
    let lit = proc_macro2::Literal::string(&result);
    proc_macro2::TokenStream::from(proc_macro2::TokenTree::Literal(lit)).into()
}

fn reconstruct(tts: &[TokenTree]) -> String {
    if tts.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    let mut prev_end: Option<LineColumn> = None;

    for tt in tts {
        let start = tt.span().start();

        // Add whitespace between tokens
        if let Some(prev) = prev_end {
            let ws = compute_whitespace(prev, start);
            result.push_str(&ws);
        }

        result.push_str(&token_to_string(tt));
        prev_end = Some(tt.span().end());
    }

    result
}

/// Compute whitespace between two positions.
/// If positions seem wrong (backwards or huge jump), just use a single space.
fn compute_whitespace(from: LineColumn, to: LineColumn) -> String {
    // Going backwards? Position is wrong, use single space.
    if to.line < from.line || (to.line == from.line && to.column < from.column) {
        return " ".to_string();
    }

    // Huge jump forward (> 4 lines)? Position is wrong, use single space.
    if to.line > from.line + 4 {
        return " ".to_string();
    }

    if from.line == to.line {
        // Same line - use spaces
        let spaces = to.column.saturating_sub(from.column);
        if spaces == 0 {
            String::new()
        } else {
            " ".repeat(spaces)
        }
    } else {
        // Different lines - newlines + indentation
        let newlines = to.line - from.line;
        let mut ws = "\n".repeat(newlines);
        ws.push_str(&" ".repeat(to.column));
        ws
    }
}

fn token_to_string(tt: &TokenTree) -> String {
    match tt {
        TokenTree::Group(g) => {
            let inner: Vec<TokenTree> = g.stream().into_iter().collect();
            let (open, close) = match g.delimiter() {
                proc_macro2::Delimiter::Parenthesis => ("(", ")"),
                proc_macro2::Delimiter::Brace => ("{", "}"),
                proc_macro2::Delimiter::Bracket => ("[", "]"),
                proc_macro2::Delimiter::None => ("", ""),
            };

            if inner.is_empty() {
                return format!("{}{}", open, close);
            }

            let group_start = g.span().start();
            let group_end = g.span().end();
            let first_start = inner.first().unwrap().span().start();
            let last_end = inner.last().unwrap().span().end();

            // Check if group span seems wrong (big jump to first inner token)
            let group_span_wrong = {
                let line_diff = if first_start.line > group_start.line {
                    first_start.line - group_start.line
                } else {
                    group_start.line - first_start.line
                };
                line_diff > 4
            };

            let leading = if group_span_wrong {
                // Group span is wrong - use newline + first token's column as indent
                if g.delimiter() == proc_macro2::Delimiter::Brace {
                    let mut ws = "\n".to_string();
                    ws.push_str(&" ".repeat(first_start.column));
                    ws
                } else {
                    String::new()
                }
            } else {
                // Group span seems ok - compute normally
                compute_whitespace(
                    LineColumn { line: group_start.line, column: group_start.column + 1 },
                    first_start,
                )
            };

            let trailing = if group_span_wrong {
                // Group span is wrong - use newline for brace, nothing for others
                if g.delimiter() == proc_macro2::Delimiter::Brace {
                    // Use last token's line to figure out closing brace indent
                    let mut ws = "\n".to_string();
                    // Closing brace should be at same indent as content minus one level
                    // Approximate: use first_start.column - 4, or 0
                    ws.push_str(&" ".repeat(first_start.column.saturating_sub(4)));
                    ws
                } else {
                    String::new()
                }
            } else {
                compute_whitespace(last_end, group_end)
            };

            let inner_str = reconstruct(&inner);
            format!("{}{}{}{}{}", open, leading, inner_str, trailing, close)
        }
        TokenTree::Ident(i) => i.to_string(),
        TokenTree::Punct(p) => p.to_string(),
        TokenTree::Literal(l) => l.to_string(),
    }
}

#[cfg(test)]
mod tests {}
