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

/// Returns the line number of the first token's span start.
/// Returns 0 if there are no tokens.
#[proc_macro]
pub fn line_of_first_token(input: TokenStream) -> TokenStream {
    let input2: proc_macro2::TokenStream = input.into();
    let line = if let Some(first) = input2.into_iter().next() {
        first.span().start().line
    } else {
        0
    };
    let lit = proc_macro2::Literal::usize_unsuffixed(line);
    proc_macro2::TokenStream::from(proc_macro2::TokenTree::Literal(lit)).into()
}

/// Returns the line number of the last token's span end.
/// Returns 0 if there are no tokens.
#[proc_macro]
pub fn line_of_last_token(input: TokenStream) -> TokenStream {
    let input2: proc_macro2::TokenStream = input.into();
    let tts: Vec<TokenTree> = input2.into_iter().collect();
    let line = if let Some(last) = tts.last() {
        last.span().end().line
    } else {
        0
    };
    let lit = proc_macro2::Literal::usize_unsuffixed(line);
    proc_macro2::TokenStream::from(proc_macro2::TokenTree::Literal(lit)).into()
}

fn reconstruct(tts: &[TokenTree]) -> String {
    if tts.is_empty() {
        return String::new();
    }

    // Use the first token's column as the base - all other columns will be relative to this
    let base_column = tts.first().map(|tt| tt.span().start().column).unwrap_or(0);

    let mut result = String::new();
    let mut prev_end: Option<LineColumn> = None;

    for tt in tts {
        let start = tt.span().start();

        // Add whitespace between tokens
        if let Some(prev) = prev_end {
            let ws = compute_whitespace_relative(prev, start, base_column);
            result.push_str(&ws);
        }

        result.push_str(&token_to_string_inner(tt, None, base_column));
        prev_end = Some(tt.span().end());
    }

    result
}

/// Compute whitespace between two positions, with columns relative to a base.
fn compute_whitespace_relative(from: LineColumn, to: LineColumn, base_column: usize) -> String {
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
        // Different lines - newlines + relative indentation
        let newlines = to.line - from.line;
        let mut ws = "\n".repeat(newlines);
        // Use column relative to base, not absolute
        let relative_indent = to.column.saturating_sub(base_column);
        // DEBUG
        if to.line == 150 {
            eprintln!("DEBUG whitespace: from {:?} to {:?}, base={}, relative_indent={}",
                from, to, base_column, relative_indent);
        }
        ws.push_str(&" ".repeat(relative_indent));
        ws
    }
}

/// Process a token tree. `fixed_indent` is Some when we're in "fixed indentation mode"
/// (parent had wrong span info), which propagates to all nested groups.
/// `base_column` is the statement's first token's column, used to compute relative indentation.
fn token_to_string_inner(tt: &TokenTree, fixed_indent: Option<usize>, base_column: usize) -> String {
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

            // Use fixed indentation mode if parent was in it, or if this group's span is wrong
            let use_fixed = fixed_indent.is_some() || group_span_wrong;

            if use_fixed && g.delimiter() == proc_macro2::Delimiter::Brace {
                // Use fixed indentation for brace groups
                let base = fixed_indent.unwrap_or(0);
                let content_indent = base + 4;
                let inner_str = reconstruct_with_indent(&inner, content_indent);
                format!(
                    "{}\n{}{}\n{}{}",
                    open,
                    " ".repeat(content_indent),
                    inner_str,
                    " ".repeat(base),
                    close
                )
            } else if use_fixed {
                // Non-brace group in fixed mode - just stringify inner contents
                let base = fixed_indent.unwrap_or(0);
                let inner_str = reconstruct_with_indent(&inner, base);
                format!("{}{}{}", open, inner_str, close)
            } else {
                // Normal span-based reconstruction with relative columns
                let leading = compute_whitespace_relative(
                    LineColumn { line: group_start.line, column: group_start.column + 1 },
                    first_start,
                    base_column,
                );

                let trailing = if g.delimiter() != proc_macro2::Delimiter::Brace
                    && last_end.line == group_end.line
                {
                    // For parens/brackets on same line, don't add trailing spaces
                    // (span info for these is often slightly off)
                    String::new()
                } else {
                    compute_whitespace_relative(last_end, group_end, base_column)
                };

                let inner_str = reconstruct_inner(&inner, base_column);
                format!("{}{}{}{}{}", open, leading, inner_str, trailing, close)
            }
        }
        TokenTree::Ident(i) => i.to_string(),
        TokenTree::Punct(p) => p.to_string(),
        TokenTree::Literal(l) => l.to_string(),
    }
}

/// Reconstruct inner token stream with a shared base_column for relative indentation
fn reconstruct_inner(tts: &[TokenTree], base_column: usize) -> String {
    if tts.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    let mut prev_end: Option<LineColumn> = None;

    for tt in tts {
        let start = tt.span().start();

        // Add whitespace between tokens
        if let Some(prev) = prev_end {
            let ws = compute_whitespace_relative(prev, start, base_column);
            result.push_str(&ws);
        }

        result.push_str(&token_to_string_inner(tt, None, base_column));
        prev_end = Some(tt.span().end());
    }

    result
}

/// Reconstruct token stream with fixed indentation (for groups with wrong spans)
fn reconstruct_with_indent(tts: &[TokenTree], base_indent: usize) -> String {
    if tts.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    let mut prev_end: Option<LineColumn> = None;

    for tt in tts {
        let start = tt.span().start();

        // Add whitespace between tokens
        if let Some(prev) = prev_end {
            if start.line > prev.line {
                // Different line - add newline and base indent
                let newlines = (start.line - prev.line).min(3);
                result.push_str(&"\n".repeat(newlines));
                result.push_str(&" ".repeat(base_indent));
            } else {
                // Same line - use column-based spacing
                let spaces = start.column.saturating_sub(prev.column);
                if spaces > 0 {
                    result.push_str(&" ".repeat(spaces));
                }
            }
        }

        result.push_str(&token_to_string_inner(tt, Some(base_indent), base_indent));
        prev_end = Some(tt.span().end());
    }

    result
}

#[cfg(test)]
mod tests {}
