//! POSIX Shell Argument Tokenizer
//!
//! This module implements a tokenizer for splitting shell command lines into arguments,
//! handling quoting and escape sequences according to POSIX shell rules.

use core::fmt;

/// A warning generated during shell tokenization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Warning {
    /// The character that triggered the warning.
    pub character: char,
    /// The byte position in the input where the character was found.
    pub position: usize,
    /// A description of the warning.
    pub message: String,
}

impl fmt::Display for Warning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "warning at position {}: {} ('{}')",
            self.position, self.message, self.character
        )
    }
}

/// An error that occurs during shell tokenization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenizeError {
    /// An unclosed single quote was encountered.
    UnclosedSingleQuote { position: usize },
    /// An unclosed double quote was encountered.
    UnclosedDoubleQuote { position: usize },
    /// A trailing backslash was encountered at end of input.
    TrailingBackslash { position: usize },
}

impl fmt::Display for TokenizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnclosedSingleQuote { position } => {
                write!(f, "unclosed single quote starting at position {position}")
            }
            Self::UnclosedDoubleQuote { position } => {
                write!(f, "unclosed double quote starting at position {position}")
            }
            Self::TrailingBackslash { position } => {
                write!(f, "trailing backslash at position {position}")
            }
        }
    }
}

impl core::error::Error for TokenizeError {}

/// The result of tokenizing a shell command line.
#[derive(Debug, Clone)]
pub struct TokenizeResult {
    /// The parsed tokens (arguments).
    pub tokens: Vec<String>,
    /// Any warnings generated during parsing.
    pub warnings: Vec<Warning>,
}

/// The internal state of the tokenizer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Normal,
    SingleQuoted,
    DoubleQuoted,
}

/// Characters that trigger warnings when unquoted.
const UNQUOTED_WARN_CHARS: &[char] = &[
    '$', '`', '|', '&', ';', '(', ')', '<', '>', '#', '*', '?', '[',
];

/// Tokenize a shell command line into arguments according to POSIX shell rules.
///
/// # Arguments
///
/// * `input` - The shell command line to tokenize.
///
/// # Returns
///
/// Returns a `Result` containing either a `TokenizeResult` with the parsed tokens
/// and any warnings, or a `TokenizeError` if the input is malformed.
///
/// # Examples
///
/// ```
/// use jeb::shell_tokenizer::tokenize;
///
/// let result = tokenize("hello world").unwrap();
/// assert_eq!(result.tokens, vec!["hello", "world"]);
/// assert!(result.warnings.is_empty());
///
/// let result = tokenize("'hello world'").unwrap();
/// assert_eq!(result.tokens, vec!["hello world"]);
///
/// let result = tokenize("hello\\ world").unwrap();
/// assert_eq!(result.tokens, vec!["hello world"]);
/// ```
#[expect(clippy::too_many_lines)]
pub fn tokenize(input: &str) -> Result<TokenizeResult, TokenizeError> {
    let mut state = State::Normal;
    let mut at_word_start = true;
    let mut current_token = String::new();
    let mut token_started = false; // Track if we've started building a token
    let mut tokens = Vec::new();
    let mut warnings = Vec::new();

    // Track the position where a quote started, for error messages
    let mut quote_start_position: Option<usize> = None;

    let chars: Vec<char> = input.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        let position = i;

        match state {
            State::SingleQuoted => {
                if c == '\'' {
                    state = State::Normal;
                    at_word_start = false;
                    quote_start_position = None;
                } else {
                    current_token.push(c);
                }
            }
            State::DoubleQuoted => {
                if c == '\\' {
                    if let Some(&next) = chars.get(i + 1) {
                        if matches!(next, '$' | '`' | '"' | '\\') {
                            current_token.push(next);
                            i += 1;
                        } else if next == '\n' {
                            // Line continuation - skip both characters
                            i += 1;
                        } else {
                            // Backslash is literal
                            current_token.push('\\');
                        }
                    } else {
                        // Backslash at end of input inside double quotes
                        current_token.push('\\');
                    }
                } else if c == '"' {
                    state = State::Normal;
                    at_word_start = false;
                    quote_start_position = None;
                } else if c == '`' {
                    warnings.push(Warning {
                        character: c,
                        position,
                        message: "backtick in double quotes (command substitution not interpreted)"
                            .to_string(),
                    });
                    current_token.push(c);
                } else if c == '$' {
                    warnings.push(Warning {
                        character: c,
                        position,
                        message:
                            "dollar sign in double quotes (variable expansion not interpreted)"
                                .to_string(),
                    });
                    current_token.push(c);
                } else {
                    current_token.push(c);
                }
            }
            State::Normal => {
                if c == '\\' {
                    if let Some(&next) = chars.get(i + 1) {
                        if next == '\n' {
                            // Line continuation - skip both characters
                            i += 1;
                        } else {
                            current_token.push(next);
                            i += 1;
                            at_word_start = false;
                        }
                    } else {
                        // Trailing backslash at EOF
                        return Err(TokenizeError::TrailingBackslash { position });
                    }
                } else if c == '\'' {
                    state = State::SingleQuoted;
                    at_word_start = false;
                    token_started = true;
                    quote_start_position = Some(position);
                } else if c == '"' {
                    state = State::DoubleQuoted;
                    at_word_start = false;
                    token_started = true;
                    quote_start_position = Some(position);
                } else if c == ' ' || c == '\t' {
                    if token_started || !current_token.is_empty() {
                        tokens.push(core::mem::take(&mut current_token));
                        token_started = false;
                    }
                    at_word_start = true;
                } else if c == '~' && at_word_start {
                    warnings.push(Warning {
                        character: c,
                        position,
                        message: "tilde at word start (tilde expansion not interpreted)"
                            .to_string(),
                    });
                    current_token.push(c);
                    at_word_start = false;
                } else if UNQUOTED_WARN_CHARS.contains(&c) {
                    let message = match c {
                        '$' => "dollar sign (variable expansion not interpreted)",
                        '`' => "backtick (command substitution not interpreted)",
                        '|' => "pipe (piping not interpreted)",
                        '&' => "ampersand (background/AND not interpreted)",
                        ';' => "semicolon (command separator not interpreted)",
                        '(' => "open parenthesis (subshell not interpreted)",
                        ')' => "close parenthesis (subshell not interpreted)",
                        '<' => "less-than (input redirection not interpreted)",
                        '>' => "greater-than (output redirection not interpreted)",
                        '#' => "hash (comment not interpreted)",
                        '*' => "asterisk (glob wildcard not interpreted)",
                        '?' => "question mark (glob wildcard not interpreted)",
                        '[' => "open bracket (glob bracket expression not interpreted)",
                        _ => "shell metacharacter not interpreted",
                    };
                    warnings.push(Warning {
                        character: c,
                        position,
                        message: message.to_string(),
                    });
                    current_token.push(c);
                    at_word_start = false;
                } else {
                    current_token.push(c);
                    at_word_start = false;
                }
            }
        }

        i += 1;
    }

    // End of input
    if token_started || !current_token.is_empty() {
        tokens.push(current_token);
    }

    match state {
        State::SingleQuoted => {
            return Err(TokenizeError::UnclosedSingleQuote {
                position: quote_start_position.unwrap_or(0),
            });
        }
        State::DoubleQuoted => {
            return Err(TokenizeError::UnclosedDoubleQuote {
                position: quote_start_position.unwrap_or(0),
            });
        }
        State::Normal => {}
    }

    Ok(TokenizeResult { tokens, warnings })
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to assert tokens without warnings
    fn assert_tokens(input: &str, expected: &[&str]) {
        let result = tokenize(input).expect("tokenization should succeed");
        assert_eq!(
            result.tokens,
            expected.iter().map(|s| (*s).to_string()).collect::<Vec<_>>()
        );
        assert!(
            result.warnings.is_empty(),
            "expected no warnings, got: {:?}",
            result.warnings
        );
    }

    // Helper to assert tokens with warnings
    fn assert_tokens_with_warnings(input: &str, expected: &[&str], warning_chars: &[char]) {
        let result = tokenize(input).expect("tokenization should succeed");
        assert_eq!(
            result.tokens,
            expected.iter().map(|s| (*s).to_string()).collect::<Vec<_>>()
        );
        let actual_warning_chars: Vec<char> = result.warnings.iter().map(|w| w.character).collect();
        assert_eq!(actual_warning_chars, warning_chars);
    }

    // Helper to assert error
    fn assert_error(input: &str, expected_error: TokenizeError) {
        let result = tokenize(input);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), expected_error);
    }

    // MARK: Basic Tokenization

    #[test]
    fn test_basic_tokenization() {
        assert_tokens("hello world", &["hello", "world"]);
        assert_tokens("hello   world", &["hello", "world"]);
        assert_tokens("  hello world  ", &["hello", "world"]);
        assert_tokens("hello", &["hello"]);
        assert_tokens("", &[]);
        assert_tokens("   ", &[]);
    }

    #[test]
    fn test_tabs() {
        assert_tokens("hello\tworld", &["hello", "world"]);
        assert_tokens("hello \t world", &["hello", "world"]);
    }

    // MARK: Single Quotes

    #[test]
    fn test_single_quotes() {
        assert_tokens("'hello world'", &["hello world"]);
        assert_tokens("'$HOME'", &["$HOME"]);
        assert_tokens("'\\n'", &["\\n"]);
        assert_tokens("'it'\\''s'", &["it's"]);
    }

    #[test]
    fn test_unclosed_single_quote() {
        assert_error(
            "'hello",
            TokenizeError::UnclosedSingleQuote { position: 0 },
        );
    }

    // MARK: Double Quotes

    #[test]
    fn test_double_quotes() {
        assert_tokens("\"hello world\"", &["hello world"]);
        assert_tokens("\"say \\\"hi\\\"\"", &["say \"hi\""]);
        assert_tokens("\"back\\\\slash\"", &["back\\slash"]);
        assert_tokens("\"\\$HOME\"", &["$HOME"]);
        assert_tokens("\"\\n\"", &["\\n"]);
        assert_tokens("\"\\z\"", &["\\z"]);
    }

    #[test]
    fn test_dollar_in_double_quotes_warns() {
        assert_tokens_with_warnings("\"$HOME\"", &["$HOME"], &['$']);
    }

    #[test]
    fn test_backtick_in_double_quotes_warns() {
        assert_tokens_with_warnings("\"`cmd`\"", &["`cmd`"], &['`', '`']);
    }

    #[test]
    fn test_unclosed_double_quote() {
        assert_error(
            "\"hello",
            TokenizeError::UnclosedDoubleQuote { position: 0 },
        );
    }

    // MARK: Unquoted Escapes

    #[test]
    fn test_unquoted_escapes() {
        assert_tokens("hello\\ world", &["hello world"]);
        assert_tokens("\\$HOME", &["$HOME"]);
        assert_tokens("\\\\", &["\\"]);
        assert_tokens("\\*", &["*"]);
    }

    #[test]
    fn test_trailing_backslash_error() {
        assert_error("hello\\", TokenizeError::TrailingBackslash { position: 5 });
    }

    #[test]
    fn test_line_continuation() {
        assert_tokens("hello\\\nworld", &["helloworld"]);
        assert_tokens("hello \\\n world", &["hello", "world"]);
    }

    #[test]
    fn test_line_continuation_in_double_quotes() {
        assert_tokens("\"hello\\\nworld\"", &["helloworld"]);
    }

    // MARK: Warnings

    #[test]
    fn test_unquoted_dollar_warns() {
        assert_tokens_with_warnings("$HOME", &["$HOME"], &['$']);
    }

    #[test]
    fn test_unquoted_glob_warns() {
        assert_tokens_with_warnings("*.txt", &["*.txt"], &['*']);
        assert_tokens_with_warnings("file?", &["file?"], &['?']);
        assert_tokens_with_warnings("file[0]", &["file[0]"], &['[']);
    }

    #[test]
    fn test_tilde_at_word_start_warns() {
        assert_tokens_with_warnings("~user", &["~user"], &['~']);
    }

    #[test]
    fn test_tilde_mid_word_no_warn() {
        assert_tokens("a~b", &["a~b"]);
    }

    #[test]
    fn test_pipe_and_semicolon_warn() {
        assert_tokens_with_warnings("echo hello|cat", &["echo", "hello|cat"], &['|']);
        assert_tokens_with_warnings("echo; ls", &["echo;", "ls"], &[';']);
    }

    #[test]
    fn test_redirections_warn() {
        assert_tokens_with_warnings("echo > file", &["echo", ">", "file"], &['>']);
        assert_tokens_with_warnings("cat < file", &["cat", "<", "file"], &['<']);
    }

    // MARK: Token Concatenation

    #[test]
    fn test_concatenation() {
        assert_tokens("a'b'c", &["abc"]);
        assert_tokens("a\"b\"c", &["abc"]);
        assert_tokens("'a'\"b\"c", &["abc"]);
        assert_tokens("x=\"foo\"", &["x=foo"]);
    }

    // MARK: Empty Quotes

    #[test]
    fn test_empty_quotes() {
        assert_tokens("''", &[""]);
        assert_tokens("\"\"", &[""]);
        assert_tokens("'' ''", &["", ""]);
    }

    #[test]
    fn test_adjacent_empty_quotes() {
        assert_tokens("a''b", &["ab"]);
        assert_tokens("a\"\"b", &["ab"]);
        assert_tokens("''\"\"", &[""]);
    }

    // MARK: Newlines in Quotes

    #[test]
    fn test_newlines_in_single_quotes() {
        assert_tokens("'hello\nworld'", &["hello\nworld"]);
    }

    #[test]
    fn test_newlines_in_double_quotes() {
        assert_tokens("\"hello\nworld\"", &["hello\nworld"]);
    }

    // MARK: Escaped Quote Characters

    #[test]
    fn test_escaped_single_quote() {
        assert_tokens("\\'", &["'"]);
    }

    #[test]
    fn test_escaped_double_quote() {
        assert_tokens("\\\"", &["\""]);
    }

    #[test]
    fn test_escaped_quote_in_double_quotes() {
        assert_tokens("\"he said \\\"hi\\\"\"", &["he said \"hi\""]);
    }

    // MARK: Complex Cases

    #[test]
    fn test_complex_concatenation() {
        assert_tokens("a\"b\"c'd'e", &["abcde"]);
    }

    #[test]
    fn test_multiple_warnings() {
        let result = tokenize("$HOME/*.txt").unwrap();
        assert_eq!(result.tokens, vec!["$HOME/*.txt"]);
        assert_eq!(result.warnings.len(), 2);
        assert_eq!(result.warnings[0].character, '$');
        assert_eq!(result.warnings[1].character, '*');
    }

    #[test]
    fn test_backslash_in_double_quotes_before_regular_char() {
        // Backslash before a non-special char in double quotes should keep both
        assert_tokens("\"\\a\"", &["\\a"]);
        assert_tokens("\"\\x\"", &["\\x"]);
    }
}
