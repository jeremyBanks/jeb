//! POSIX Shell Argument Tokenizer
//!
//! This module implements a tokenizer for splitting shell command lines into
//! arguments, handling quoting and escape sequences according to POSIX shell
//! rules.
//!
//! This tokenizer produces correct results for valid inputs that only use
//! single-quoted strings, double-quoted strings, and backslash escapes. If
//! unsupported shell syntax is encountered (such as variable expansion, command
//! substitution, globs, or other shell features), the tokenizer produces a
//! best-effort result but populates the `errors` list in the result, indicating
//! that the output should not be trusted.

/// The kind of error encountered during shell tokenization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    /// An unclosed single quote was encountered.
    UnclosedSingleQuote,
    /// An unclosed double quote was encountered.
    UnclosedDoubleQuote,
    /// A trailing backslash was encountered at end of input.
    TrailingBackslash,
    /// Dollar sign for variable expansion (not interpreted).
    DollarSign,
    /// Backtick for command substitution (not interpreted).
    Backtick,
    /// Pipe for piping (not interpreted).
    Pipe,
    /// Ampersand for background/AND (not interpreted).
    Ampersand,
    /// Semicolon as command separator (not interpreted).
    Semicolon,
    /// Newline as command separator (not interpreted).
    Newline,
    /// Open parenthesis for subshell (not interpreted).
    OpenParen,
    /// Close parenthesis for subshell (not interpreted).
    CloseParen,
    /// Less-than for input redirection (not interpreted).
    LessThan,
    /// Greater-than for output redirection (not interpreted).
    GreaterThan,
    /// Hash for comment (not interpreted).
    Hash,
    /// Asterisk glob wildcard (not interpreted).
    Asterisk,
    /// Question mark glob wildcard (not interpreted).
    QuestionMark,
    /// Open bracket for glob bracket expression (not interpreted).
    OpenBracket,
    /// Tilde at word start for tilde expansion (not interpreted).
    Tilde,
}

impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnclosedSingleQuote => write!(f, "unclosed single quote"),
            Self::UnclosedDoubleQuote => write!(f, "unclosed double quote"),
            Self::TrailingBackslash => write!(f, "trailing backslash"),
            Self::DollarSign => write!(f, "dollar sign (variable expansion not interpreted)"),
            Self::Backtick => write!(f, "backtick (command substitution not interpreted)"),
            Self::Pipe => write!(f, "pipe (piping not interpreted)"),
            Self::Ampersand => write!(f, "ampersand (background/AND not interpreted)"),
            Self::Semicolon => write!(f, "semicolon (command separator not interpreted)"),
            Self::Newline => write!(
                f,
                "newline (command separator interpreted as whitespace instead)"
            ),
            Self::OpenParen => write!(f, "open parenthesis (subshell not interpreted)"),
            Self::CloseParen => write!(f, "close parenthesis (subshell not interpreted)"),
            Self::LessThan => write!(f, "less-than (input redirection not interpreted)"),
            Self::GreaterThan => write!(f, "greater-than (output redirection not interpreted)"),
            Self::Hash => write!(f, "hash (comment not interpreted)"),
            Self::Asterisk => write!(f, "asterisk (glob wildcard not interpreted)"),
            Self::QuestionMark => write!(f, "question mark (glob wildcard not interpreted)"),
            Self::OpenBracket => {
                write!(f, "open bracket (glob bracket expression not interpreted)")
            }
            Self::Tilde => write!(f, "tilde (tilde expansion not interpreted)"),
        }
    }
}

/// An error encountered during shell tokenization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    /// The kind of error.
    pub kind: ErrorKind,
    /// The byte that triggered the error.
    pub byte: u8,
    /// The byte position in the input where the error occurred.
    pub position: usize,
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "error at position {}: {} (byte 0x{:02x})",
            self.position, self.kind, self.byte
        )
    }
}

/// The result of tokenizing a shell command line.
///
/// If `errors` is non-empty, the `args` should not be trusted as they may be
/// incorrect due to unsupported shell syntax being encountered.
#[derive(Debug, Clone)]
pub struct TokenizeResult {
    /// The parsed arguments. If `errors` is non-empty, these may be incorrect.
    pub args: Vec<Vec<u8>>,
    /// Errors encountered during parsing. If non-empty, the args may be
    /// incorrect.
    pub errors: Vec<Error>,
}

/// The internal state of the tokenizer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Normal,
    SingleQuoted,
    DoubleQuoted,
}

/// Bytes that trigger errors when unquoted.
const UNQUOTED_WARN_BYTES: &[u8] = b"$`|&;()<>*?[";

/// Tokenize a shell command line (as bytes) into arguments according to POSIX
/// shell rules.
///
/// This function produces correct results for valid inputs that only use
/// single-quoted strings, double-quoted strings, and backslash escapes. If
/// unsupported shell syntax is encountered, the function produces a best-effort
/// result but populates the `errors` list, indicating that the output should
/// not be trusted.
///
/// # Arguments
///
/// * `input` - The shell command line as bytes.
///
/// # Returns
///
/// Returns a `TokenizeResult` with the parsed arguments and any errors
/// encountered. If `errors` is non-empty, the `args` may be incorrect.
#[expect(clippy::too_many_lines)]
#[must_use]
pub fn tokenize(input: &[u8]) -> TokenizeResult {
    let mut state = State::Normal;
    let mut at_word_start = true;
    let mut current_token = Vec::<u8>::new();
    let mut token_started = false;
    let mut args = Vec::new();
    let mut errors = Vec::new();

    // Track the position where a quote started, for error messages
    let mut quote_start_position: Option<usize> = None;

    let mut i = 0;

    while i < input.len() {
        let b = input[i];
        let position = i;

        match state {
            State::Normal => {
                if b == b'\\' {
                    if let Some(&next) = input.get(i + 1) {
                        if next == b'\n' {
                            // Line continuation - skip both bytes
                            i += 1;
                        } else {
                            current_token.push(next);
                            i += 1;
                            at_word_start = false;
                        }
                    } else {
                        errors.push(Error {
                            kind: ErrorKind::TrailingBackslash,
                            byte: b,
                            position,
                        });
                    }
                } else if b == b'\'' {
                    state = State::SingleQuoted;
                    at_word_start = false;
                    token_started = true;
                    quote_start_position = Some(position);
                } else if b == b'"' {
                    state = State::DoubleQuoted;
                    at_word_start = false;
                    token_started = true;
                    quote_start_position = Some(position);
                } else if b == b' ' || b == b'\t' {
                    if token_started || !current_token.is_empty() {
                        args.push(core::mem::take(&mut current_token));
                        token_started = false;
                    }
                    at_word_start = true;
                } else if b == b'\n' || b == b'\r' {
                    errors.push(Error {
                        kind: ErrorKind::Newline,
                        byte: b,
                        position,
                    });
                    if token_started || !current_token.is_empty() {
                        args.push(core::mem::take(&mut current_token));
                        token_started = false;
                    }
                    at_word_start = true;
                } else if b == b'~' && at_word_start {
                    errors.push(Error {
                        kind: ErrorKind::Tilde,
                        byte: b,
                        position,
                    });
                    current_token.push(b);
                    at_word_start = false;
                } else if b == b'#' && at_word_start {
                    errors.push(Error {
                        kind: ErrorKind::Hash,
                        byte: b,
                        position,
                    });
                    current_token.push(b);
                    at_word_start = false;
                } else if UNQUOTED_WARN_BYTES.contains(&b) {
                    let kind = match b {
                        b'$' => ErrorKind::DollarSign,
                        b'`' => ErrorKind::Backtick,
                        b'|' => ErrorKind::Pipe,
                        b'&' => ErrorKind::Ampersand,
                        b';' => ErrorKind::Semicolon,
                        b'(' => ErrorKind::OpenParen,
                        b')' => ErrorKind::CloseParen,
                        b'<' => ErrorKind::LessThan,
                        b'>' => ErrorKind::GreaterThan,
                        b'*' => ErrorKind::Asterisk,
                        b'?' => ErrorKind::QuestionMark,
                        b'[' => ErrorKind::OpenBracket,
                        _ => unreachable!(),
                    };
                    errors.push(Error {
                        kind,
                        byte: b,
                        position,
                    });
                    current_token.push(b);
                    at_word_start = false;
                } else {
                    current_token.push(b);
                    at_word_start = false;
                }
            }
            State::SingleQuoted => {
                if b == b'\'' {
                    state = State::Normal;
                    at_word_start = false;
                    quote_start_position = None;
                } else {
                    current_token.push(b);
                }
            }
            State::DoubleQuoted => {
                if b == b'\\' {
                    if let Some(&next) = input.get(i + 1) {
                        if matches!(next, b'$' | b'`' | b'"' | b'\\') {
                            current_token.push(next);
                            i += 1;
                        } else if next == b'\n' {
                            // Line continuation - skip both bytes
                            i += 1;
                        } else {
                            current_token.push(b'\\');
                        }
                    } else {
                        current_token.push(b'\\');
                    }
                } else if b == b'"' {
                    state = State::Normal;
                    at_word_start = false;
                    quote_start_position = None;
                } else if b == b'`' {
                    errors.push(Error {
                        kind: ErrorKind::Backtick,
                        byte: b,
                        position,
                    });
                    current_token.push(b);
                } else if b == b'$' {
                    errors.push(Error {
                        kind: ErrorKind::DollarSign,
                        byte: b,
                        position,
                    });
                    current_token.push(b);
                } else {
                    current_token.push(b);
                }
            }
        }

        i += 1;
    }

    if token_started || !current_token.is_empty() {
        args.push(current_token);
    }

    match state {
        State::Normal => {}
        State::SingleQuoted => {
            errors.push(Error {
                kind: ErrorKind::UnclosedSingleQuote,
                byte: b'\'',
                position: quote_start_position.unwrap_or(0),
            });
        }
        State::DoubleQuoted => {
            errors.push(Error {
                kind: ErrorKind::UnclosedDoubleQuote,
                byte: b'"',
                position: quote_start_position.unwrap_or(0),
            });
        }
    }

    TokenizeResult { args, errors }
}

/// Tokenize a shell command line string into arguments according to POSIX shell
/// rules.
///
/// This is a convenience wrapper around [`tokenize`] that works with `&str`
/// input and produces `String` output. Since the tokenizer only operates on
/// ASCII control characters, it preserves UTF-8 validity.
///
/// # Arguments
///
/// * `input` - The shell command line as a string.
///
/// # Returns
///
/// Returns a tuple of (args, errors) where args are the parsed arguments as
/// strings and errors are any errors encountered. If errors is non-empty, the
/// args may be incorrect.
///
/// # Examples
///
/// ```
/// use jeb::shell_tokenizer::tokenize_str;
///
/// let (args, errors) = tokenize_str("hello world");
/// assert_eq!(args, vec!["hello", "world"]);
/// assert!(errors.is_empty());
///
/// let (args, errors) = tokenize_str("'hello world'");
/// assert_eq!(args, vec!["hello world"]);
///
/// let (args, errors) = tokenize_str("hello\\ world");
/// assert_eq!(args, vec!["hello world"]);
/// ```
#[must_use]
pub fn tokenize_str(input: &str) -> (Vec<String>, Vec<Error>) {
    let result = tokenize(input.as_bytes());
    let args = result
        .args
        .into_iter()
        .map(|bytes| String::from_utf8(bytes).expect("tokenizer should preserve UTF-8 validity"))
        .collect();
    (args, result.errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to assert args without errors using tokenize_str
    fn assert_args_str(input: &str, expected: &[&str]) {
        let (args, errors) = tokenize_str(input);
        assert_eq!(
            args,
            expected
                .iter()
                .map(|s| (*s).to_string())
                .collect::<Vec<_>>()
        );
        assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
    }

    // Helper to assert args without errors using tokenize (bytes)
    fn assert_args(input: &[u8], expected: &[&[u8]]) {
        let result = tokenize(input);
        assert_eq!(
            result.args,
            expected.iter().map(|s| s.to_vec()).collect::<Vec<_>>()
        );
        assert!(
            result.errors.is_empty(),
            "expected no errors, got: {:?}",
            result.errors
        );
    }

    // Helper to assert args with errors using tokenize_str
    fn assert_args_with_errors_str(input: &str, expected: &[&str], error_bytes: &[u8]) {
        let (args, errors) = tokenize_str(input);
        assert_eq!(
            args,
            expected
                .iter()
                .map(|s| (*s).to_string())
                .collect::<Vec<_>>()
        );
        let actual_error_bytes: Vec<u8> = errors.iter().map(|e| e.byte).collect();
        assert_eq!(actual_error_bytes, error_bytes);
    }

    // Helper to assert args with errors using tokenize (bytes)
    fn assert_args_with_errors(input: &[u8], expected: &[&[u8]], error_bytes: &[u8]) {
        let result = tokenize(input);
        assert_eq!(
            result.args,
            expected.iter().map(|s| s.to_vec()).collect::<Vec<_>>()
        );
        let actual_error_bytes: Vec<u8> = result.errors.iter().map(|e| e.byte).collect();
        assert_eq!(actual_error_bytes, error_bytes);
    }

    // Helper to assert specific error kind
    fn assert_has_error(input: &str, expected_kind: ErrorKind) {
        let (_, errors) = tokenize_str(input);
        assert!(
            errors.iter().any(|e| e.kind == expected_kind),
            "expected error {expected_kind:?}, got: {errors:?}"
        );
    }

    // MARK: Basic Tokenization (using tokenize_str)

    #[test]
    fn test_basic_tokenization() {
        assert_args_str("hello world", &["hello", "world"]);
        assert_args_str("hello   world", &["hello", "world"]);
        assert_args_str("  hello world  ", &["hello", "world"]);
        assert_args_str("hello", &["hello"]);
        assert_args_str("", &[]);
        assert_args_str("   ", &[]);
    }

    #[test]
    fn test_tabs() {
        assert_args_str("hello\tworld", &["hello", "world"]);
        assert_args_str("hello \t world", &["hello", "world"]);
    }

    // MARK: Basic Tokenization (using tokenize with bytes)

    #[test]
    fn test_basic_tokenization_bytes() {
        assert_args(b"hello world", &[b"hello", b"world"]);
        assert_args(b"hello   world", &[b"hello", b"world"]);
        assert_args(b"  hello world  ", &[b"hello", b"world"]);
        assert_args(b"hello", &[b"hello"]);
        assert_args(b"", &[]);
        assert_args(b"   ", &[]);
    }

    // MARK: Single Quotes

    #[test]
    fn test_single_quotes() {
        assert_args_str("'hello world'", &["hello world"]);
        assert_args_str("'$HOME'", &["$HOME"]);
        assert_args_str("'\\n'", &["\\n"]);
        assert_args_str("'it'\\''s'", &["it's"]);
    }

    #[test]
    fn test_unclosed_single_quote() {
        assert_has_error("'hello", ErrorKind::UnclosedSingleQuote);
    }

    // MARK: Double Quotes

    #[test]
    fn test_double_quotes() {
        assert_args_str("\"hello world\"", &["hello world"]);
        assert_args_str("\"say \\\"hi\\\"\"", &["say \"hi\""]);
        assert_args_str("\"back\\\\slash\"", &["back\\slash"]);
        assert_args_str("\"\\$HOME\"", &["$HOME"]);
        assert_args_str("\"\\n\"", &["\\n"]);
        assert_args_str("\"\\z\"", &["\\z"]);
    }

    #[test]
    fn test_dollar_in_double_quotes_warns() {
        assert_args_with_errors_str("\"$HOME\"", &["$HOME"], b"$");
    }

    #[test]
    fn test_backtick_in_double_quotes_warns() {
        assert_args_with_errors_str("\"`cmd`\"", &["`cmd`"], b"``");
    }

    #[test]
    fn test_unclosed_double_quote() {
        assert_has_error("\"hello", ErrorKind::UnclosedDoubleQuote);
    }

    // MARK: Unquoted Escapes

    #[test]
    fn test_unquoted_escapes() {
        assert_args_str("hello\\ world", &["hello world"]);
        assert_args_str("\\$HOME", &["$HOME"]);
        assert_args_str("\\\\", &["\\"]);
        assert_args_str("\\*", &["*"]);
    }

    #[test]
    fn test_trailing_backslash_error() {
        assert_has_error("hello\\", ErrorKind::TrailingBackslash);
    }

    #[test]
    fn test_line_continuation() {
        assert_args_str("hello\\\nworld", &["helloworld"]);
        assert_args_str("hello \\\n world", &["hello", "world"]);
    }

    #[test]
    fn test_line_continuation_in_double_quotes() {
        assert_args_str("\"hello\\\nworld\"", &["helloworld"]);
    }

    // MARK: Errors

    #[test]
    fn test_unquoted_dollar_warns() {
        assert_args_with_errors_str("$HOME", &["$HOME"], b"$");
    }

    #[test]
    fn test_unquoted_glob_warns() {
        assert_args_with_errors_str("*.txt", &["*.txt"], b"*");
        assert_args_with_errors_str("file?", &["file?"], b"?");
        assert_args_with_errors_str("file[0]", &["file[0]"], b"[");
    }

    #[test]
    fn test_tilde_at_word_start_warns() {
        assert_args_with_errors_str("~user", &["~user"], b"~");
    }

    #[test]
    fn test_tilde_mid_word_no_warn() {
        assert_args_str("a~b", &["a~b"]);
    }

    #[test]
    fn test_hash_at_word_start_warns() {
        assert_args_with_errors_str("#comment", &["#comment"], b"#");
        assert_args_with_errors_str("echo #test", &["echo", "#test"], b"#");
    }

    #[test]
    fn test_hash_mid_word_no_warn() {
        assert_args_str("foo#bar", &["foo#bar"]);
        assert_args_str("C#", &["C#"]);
    }

    #[test]
    fn test_pipe_and_semicolon_warn() {
        assert_args_with_errors_str("echo hello|cat", &["echo", "hello|cat"], b"|");
        assert_args_with_errors_str("echo; ls", &["echo;", "ls"], b";");
    }

    #[test]
    fn test_redirections_warn() {
        assert_args_with_errors_str("echo > file", &["echo", ">", "file"], b">");
        assert_args_with_errors_str("cat < file", &["cat", "<", "file"], b"<");
    }

    // MARK: Token Concatenation

    #[test]
    fn test_concatenation() {
        assert_args_str("a'b'c", &["abc"]);
        assert_args_str("a\"b\"c", &["abc"]);
        assert_args_str("'a'\"b\"c", &["abc"]);
        assert_args_str("x=\"foo\"", &["x=foo"]);
    }

    // MARK: Empty Quotes

    #[test]
    fn test_empty_quotes() {
        assert_args_str("''", &[""]);
        assert_args_str("\"\"", &[""]);
        assert_args_str("'' ''", &["", ""]);
    }

    #[test]
    fn test_adjacent_empty_quotes() {
        assert_args_str("a''b", &["ab"]);
        assert_args_str("a\"\"b", &["ab"]);
        assert_args_str("''\"\"", &[""]);
    }

    // MARK: Newlines in Quotes

    #[test]
    fn test_newlines_in_single_quotes() {
        assert_args_str("'hello\nworld'", &["hello\nworld"]);
    }

    #[test]
    fn test_newlines_in_double_quotes() {
        assert_args_str("\"hello\nworld\"", &["hello\nworld"]);
    }

    // MARK: Escaped Quote Characters

    #[test]
    fn test_escaped_single_quote() {
        assert_args_str("\\'", &["'"]);
    }

    #[test]
    fn test_escaped_double_quote() {
        assert_args_str("\\\"", &["\""]);
    }

    #[test]
    fn test_escaped_quote_in_double_quotes() {
        assert_args_str("\"he said \\\"hi\\\"\"", &["he said \"hi\""]);
    }

    // MARK: Complex Cases

    #[test]
    fn test_complex_concatenation() {
        assert_args_str("a\"b\"c'd'e", &["abcde"]);
    }

    #[test]
    fn test_multiple_warnings() {
        let (args, errors) = tokenize_str("$HOME/*.txt");
        assert_eq!(args, vec!["$HOME/*.txt"]);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].byte, b'$');
        assert_eq!(errors[1].byte, b'*');
    }

    #[test]
    fn test_backslash_in_double_quotes_before_regular_char() {
        assert_args_str("\"\\a\"", &["\\a"]);
        assert_args_str("\"\\x\"", &["\\x"]);
    }

    // MARK: Bytes API tests

    #[test]
    fn test_bytes_with_errors() {
        assert_args_with_errors(b"$HOME", &[b"$HOME"], b"$");
    }

    #[test]
    fn test_bytes_unclosed_quote() {
        let result = tokenize(b"'hello");
        assert!(
            result
                .errors
                .iter()
                .any(|e| e.kind == ErrorKind::UnclosedSingleQuote)
        );
    }
}
