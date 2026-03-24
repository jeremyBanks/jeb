use crate::span::Span;
use crate::token::{
    Token,
    TokenKind,
};

pub struct Lexer<'a> {
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            pos: 0,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace_and_comments();
            if self.pos >= self.bytes.len() {
                tokens.push(Token {
                    kind: TokenKind::Eof,
                    span: Span::new(self.pos, self.pos),
                    text: String::new(),
                });
                break;
            }
            tokens.push(self.next_token()?);
        }
        Ok(tokens)
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            // Skip whitespace
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_whitespace() {
                self.pos += 1;
            }
            // Skip line comments
            if self.pos + 1 < self.bytes.len()
                && self.bytes[self.pos] == b'/'
                && self.bytes[self.pos + 1] == b'/'
            {
                while self.pos < self.bytes.len() && self.bytes[self.pos] != b'\n' {
                    self.pos += 1;
                }
                continue;
            }
            // Skip block comments
            if self.pos + 1 < self.bytes.len()
                && self.bytes[self.pos] == b'/'
                && self.bytes[self.pos + 1] == b'*'
            {
                self.pos += 2;
                let mut depth = 1;
                while self.pos + 1 < self.bytes.len() && depth > 0 {
                    if self.bytes[self.pos] == b'/' && self.bytes[self.pos + 1] == b'*' {
                        depth += 1;
                        self.pos += 2;
                    } else if self.bytes[self.pos] == b'*' && self.bytes[self.pos + 1] == b'/' {
                        depth -= 1;
                        self.pos += 2;
                    } else {
                        self.pos += 1;
                    }
                }
                continue;
            }
            break;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<u8> {
        self.bytes.get(self.pos + offset).copied()
    }

    fn advance(&mut self) -> u8 {
        let b = self.bytes[self.pos];
        self.pos += 1;
        b
    }

    fn next_token(&mut self) -> Result<Token, LexError> {
        let start = self.pos;
        let b = self.peek().unwrap();

        // String literal
        if b == b'"' {
            return self.lex_string(start);
        }

        // Char literal
        if b == b'\'' {
            return self.lex_char(start);
        }

        // Number literal
        if b.is_ascii_digit() {
            return self.lex_number(start);
        }

        // Identifier or keyword
        if b.is_ascii_alphabetic() || b == b'_' {
            return Ok(self.lex_ident(start));
        }

        // Operators and punctuation
        self.lex_operator(start)
    }

    fn lex_string(&mut self, start: usize) -> Result<Token, LexError> {
        self.advance(); // skip opening "
        let mut value = String::new();
        loop {
            if self.pos >= self.bytes.len() {
                return Err(LexError {
                    pos: start,
                    msg: "unterminated string literal".into(),
                });
            }
            let b = self.advance();
            if b == b'"' {
                break;
            }
            if b == b'\\' {
                if self.pos >= self.bytes.len() {
                    return Err(LexError {
                        pos: self.pos,
                        msg: "unterminated escape sequence".into(),
                    });
                }
                let escaped = self.advance();
                match escaped {
                    b'n' => value.push('\n'),
                    b'r' => value.push('\r'),
                    b't' => value.push('\t'),
                    b'\\' => value.push('\\'),
                    b'"' => value.push('"'),
                    b'0' => value.push('\0'),
                    _ => {
                        return Err(LexError {
                            pos: self.pos - 1,
                            msg: format!("unknown escape: \\{}", escaped as char),
                        });
                    }
                }
            } else {
                value.push(b as char);
            }
        }
        let text = self.source[start..self.pos].to_string();
        Ok(Token {
            kind: TokenKind::StringLiteral(value),
            span: Span::new(start, self.pos),
            text,
        })
    }

    fn lex_char(&mut self, start: usize) -> Result<Token, LexError> {
        self.advance(); // skip opening '
        if self.pos >= self.bytes.len() {
            return Err(LexError {
                pos: start,
                msg: "unterminated char literal".into(),
            });
        }
        let ch = if self.bytes[self.pos] == b'\\' {
            self.advance();
            if self.pos >= self.bytes.len() {
                return Err(LexError {
                    pos: self.pos,
                    msg: "unterminated escape in char literal".into(),
                });
            }
            let escaped = self.advance();
            match escaped {
                b'n' => '\n',
                b'r' => '\r',
                b't' => '\t',
                b'\\' => '\\',
                b'\'' => '\'',
                b'0' => '\0',
                _ => {
                    return Err(LexError {
                        pos: self.pos - 1,
                        msg: format!("unknown escape: \\{}", escaped as char),
                    });
                }
            }
        } else {
            self.advance() as char
        };

        if self.pos >= self.bytes.len() || self.bytes[self.pos] != b'\'' {
            return Err(LexError {
                pos: self.pos,
                msg: "unterminated char literal".into(),
            });
        }
        self.advance(); // skip closing '
        let text = self.source[start..self.pos].to_string();
        Ok(Token {
            kind: TokenKind::CharLiteral(ch),
            span: Span::new(start, self.pos),
            text,
        })
    }

    fn lex_number(&mut self, start: usize) -> Result<Token, LexError> {
        // Check for 0x, 0o, 0b prefixes
        if self.bytes[self.pos] == b'0' && self.pos + 1 < self.bytes.len() {
            match self.bytes[self.pos + 1] {
                b'x' | b'X' => {
                    self.pos += 2;
                    while self.pos < self.bytes.len()
                        && (self.bytes[self.pos].is_ascii_hexdigit()
                            || self.bytes[self.pos] == b'_')
                    {
                        self.pos += 1;
                    }
                    let text = self.source[start..self.pos].to_string();
                    let digits: String = text[2..].chars().filter(|c| *c != '_').collect();
                    let value = u64::from_str_radix(&digits, 16).map_err(|_| LexError {
                        pos: start,
                        msg: "invalid hex literal".into(),
                    })?;
                    return Ok(Token {
                        kind: TokenKind::IntLiteral(value),
                        span: Span::new(start, self.pos),
                        text,
                    });
                }
                b'o' | b'O' => {
                    self.pos += 2;
                    while self.pos < self.bytes.len()
                        && (self.bytes[self.pos].is_ascii_digit() || self.bytes[self.pos] == b'_')
                    {
                        self.pos += 1;
                    }
                    let text = self.source[start..self.pos].to_string();
                    let digits: String = text[2..].chars().filter(|c| *c != '_').collect();
                    let value = u64::from_str_radix(&digits, 8).map_err(|_| LexError {
                        pos: start,
                        msg: "invalid octal literal".into(),
                    })?;
                    return Ok(Token {
                        kind: TokenKind::IntLiteral(value),
                        span: Span::new(start, self.pos),
                        text,
                    });
                }
                b'b' | b'B' => {
                    self.pos += 2;
                    while self.pos < self.bytes.len()
                        && (self.bytes[self.pos] == b'0'
                            || self.bytes[self.pos] == b'1'
                            || self.bytes[self.pos] == b'_')
                    {
                        self.pos += 1;
                    }
                    let text = self.source[start..self.pos].to_string();
                    let digits: String = text[2..].chars().filter(|c| *c != '_').collect();
                    let value = u64::from_str_radix(&digits, 2).map_err(|_| LexError {
                        pos: start,
                        msg: "invalid binary literal".into(),
                    })?;
                    return Ok(Token {
                        kind: TokenKind::IntLiteral(value),
                        span: Span::new(start, self.pos),
                        text,
                    });
                }
                _ => {}
            }
        }

        // Decimal integer or float
        while self.pos < self.bytes.len()
            && (self.bytes[self.pos].is_ascii_digit() || self.bytes[self.pos] == b'_')
        {
            self.pos += 1;
        }

        let mut is_float = false;

        // Check for decimal point (but not ..)
        if self.pos < self.bytes.len()
            && self.bytes[self.pos] == b'.'
            && self.peek_at(1) != Some(b'.')
            && self.peek_at(1).map_or(true, |b| b.is_ascii_digit())
        {
            is_float = true;
            self.pos += 1; // skip .
            while self.pos < self.bytes.len()
                && (self.bytes[self.pos].is_ascii_digit() || self.bytes[self.pos] == b'_')
            {
                self.pos += 1;
            }
        }

        // Check for exponent
        if self.pos < self.bytes.len() && (self.bytes[self.pos] == b'e' || self.bytes[self.pos] == b'E') {
            is_float = true;
            self.pos += 1;
            if self.pos < self.bytes.len()
                && (self.bytes[self.pos] == b'+' || self.bytes[self.pos] == b'-')
            {
                self.pos += 1;
            }
            while self.pos < self.bytes.len()
                && (self.bytes[self.pos].is_ascii_digit() || self.bytes[self.pos] == b'_')
            {
                self.pos += 1;
            }
        }

        // Check for type suffix (i32, i64, f32, f64, u32, u64, usize, isize)
        let suffix_start = self.pos;
        if self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_alphabetic() {
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_alphanumeric() {
                self.pos += 1;
            }
            let suffix = &self.source[suffix_start..self.pos];
            match suffix {
                "f32" | "f64" => is_float = true,
                "i32" | "i64" | "u32" | "u64" | "usize" | "isize" => {}
                _ => {
                    // Not a numeric suffix, roll back
                    self.pos = suffix_start;
                }
            }
        }

        let text = self.source[start..self.pos].to_string();
        let digits: String = text
            .chars()
            .filter(|c| *c != '_')
            .take_while(|c| c.is_ascii_digit() || *c == '.' || *c == 'e' || *c == 'E' || *c == '+' || *c == '-')
            .collect();

        if is_float {
            let value: f64 = digits.parse().map_err(|_| LexError {
                pos: start,
                msg: "invalid float literal".into(),
            })?;
            Ok(Token {
                kind: TokenKind::FloatLiteral(value),
                span: Span::new(start, self.pos),
                text,
            })
        } else {
            let value: u64 = digits.parse().map_err(|_| LexError {
                pos: start,
                msg: "invalid integer literal".into(),
            })?;
            Ok(Token {
                kind: TokenKind::IntLiteral(value),
                span: Span::new(start, self.pos),
                text,
            })
        }
    }

    fn lex_ident(&mut self, start: usize) -> Token {
        while self.pos < self.bytes.len()
            && (self.bytes[self.pos].is_ascii_alphanumeric() || self.bytes[self.pos] == b'_')
        {
            self.pos += 1;
        }
        let text = self.source[start..self.pos].to_string();
        let kind = if text == "_" {
            TokenKind::Underscore
        } else {
            TokenKind::keyword_from_str(&text).unwrap_or(TokenKind::Ident)
        };
        Token {
            kind,
            span: Span::new(start, self.pos),
            text,
        }
    }

    fn lex_operator(&mut self, start: usize) -> Result<Token, LexError> {
        let b = self.advance();
        let kind = match b {
            b'(' => TokenKind::LParen,
            b')' => TokenKind::RParen,
            b'[' => TokenKind::LBracket,
            b']' => TokenKind::RBracket,
            b'{' => TokenKind::LBrace,
            b'}' => TokenKind::RBrace,
            b',' => TokenKind::Comma,
            b';' => TokenKind::Semi,
            b'#' => TokenKind::Hash,
            b'?' => TokenKind::Question,
            b'+' => {
                if self.peek() == Some(b'=') {
                    self.advance();
                    TokenKind::PlusEq
                } else {
                    TokenKind::Plus
                }
            }
            b'-' => {
                if self.peek() == Some(b'>') {
                    self.advance();
                    TokenKind::Arrow
                } else if self.peek() == Some(b'=') {
                    self.advance();
                    TokenKind::MinusEq
                } else {
                    TokenKind::Minus
                }
            }
            b'*' => {
                if self.peek() == Some(b'=') {
                    self.advance();
                    TokenKind::StarEq
                } else {
                    TokenKind::Star
                }
            }
            b'/' => {
                if self.peek() == Some(b'=') {
                    self.advance();
                    TokenKind::SlashEq
                } else {
                    TokenKind::Slash
                }
            }
            b'%' => {
                if self.peek() == Some(b'=') {
                    self.advance();
                    TokenKind::PercentEq
                } else {
                    TokenKind::Percent
                }
            }
            b'!' => {
                if self.peek() == Some(b'=') {
                    self.advance();
                    TokenKind::NotEq
                } else {
                    TokenKind::Bang
                }
            }
            b'&' => {
                if self.peek() == Some(b'&') {
                    self.advance();
                    TokenKind::AmpAmp
                } else if self.peek() == Some(b'=') {
                    self.advance();
                    TokenKind::AmpEq
                } else {
                    TokenKind::Amp
                }
            }
            b'|' => {
                if self.peek() == Some(b'|') {
                    self.advance();
                    TokenKind::PipePipe
                } else if self.peek() == Some(b'=') {
                    self.advance();
                    TokenKind::PipeEq
                } else {
                    TokenKind::Pipe
                }
            }
            b'^' => {
                if self.peek() == Some(b'=') {
                    self.advance();
                    TokenKind::CaretEq
                } else {
                    TokenKind::Caret
                }
            }
            b'<' => {
                if self.peek() == Some(b'<') {
                    self.advance();
                    if self.peek() == Some(b'=') {
                        self.advance();
                        TokenKind::ShlEq
                    } else {
                        TokenKind::Shl
                    }
                } else if self.peek() == Some(b'=') {
                    self.advance();
                    TokenKind::LtEq
                } else {
                    TokenKind::Lt
                }
            }
            b'>' => {
                if self.peek() == Some(b'>') {
                    self.advance();
                    if self.peek() == Some(b'=') {
                        self.advance();
                        TokenKind::ShrEq
                    } else {
                        TokenKind::Shr
                    }
                } else if self.peek() == Some(b'=') {
                    self.advance();
                    TokenKind::GtEq
                } else {
                    TokenKind::Gt
                }
            }
            b'=' => {
                if self.peek() == Some(b'=') {
                    self.advance();
                    TokenKind::EqEq
                } else if self.peek() == Some(b'>') {
                    self.advance();
                    TokenKind::FatArrow
                } else {
                    TokenKind::Eq
                }
            }
            b'.' => {
                if self.peek() == Some(b'.') {
                    self.advance();
                    if self.peek() == Some(b'=') {
                        self.advance();
                        TokenKind::DotDotEq
                    } else {
                        TokenKind::DotDot
                    }
                } else {
                    TokenKind::Dot
                }
            }
            b':' => {
                if self.peek() == Some(b':') {
                    self.advance();
                    TokenKind::ColonColon
                } else {
                    TokenKind::Colon
                }
            }
            _ => {
                return Err(LexError {
                    pos: start,
                    msg: format!("unexpected character: '{}'", b as char),
                });
            }
        };

        let text = self.source[start..self.pos].to_string();
        Ok(Token {
            kind,
            span: Span::new(start, self.pos),
            text,
        })
    }
}

#[derive(Debug)]
pub struct LexError {
    pub pos: usize,
    pub msg: String,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lex error at byte {}: {}", self.pos, self.msg)
    }
}

impl std::error::Error for LexError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let mut lexer = Lexer::new("fn main() { }");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Fn);
        assert_eq!(tokens[1].kind, TokenKind::Ident);
        assert_eq!(tokens[1].text, "main");
        assert_eq!(tokens[2].kind, TokenKind::LParen);
        assert_eq!(tokens[3].kind, TokenKind::RParen);
        assert_eq!(tokens[4].kind, TokenKind::LBrace);
        assert_eq!(tokens[5].kind, TokenKind::RBrace);
        assert_eq!(tokens[6].kind, TokenKind::Eof);
    }

    #[test]
    fn test_number_literals() {
        let mut lexer = Lexer::new("42 3.14 0xFF 0b1010");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::IntLiteral(42));
        assert_eq!(tokens[1].kind, TokenKind::FloatLiteral(3.14));
        assert_eq!(tokens[2].kind, TokenKind::IntLiteral(255));
        assert_eq!(tokens[3].kind, TokenKind::IntLiteral(10));
    }

    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new(r#""hello\nworld""#);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens[0].kind,
            TokenKind::StringLiteral("hello\nworld".into())
        );
    }

    #[test]
    fn test_operators() {
        let mut lexer = Lexer::new("+ - * / == != <= >= && || -> =>");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Plus);
        assert_eq!(tokens[1].kind, TokenKind::Minus);
        assert_eq!(tokens[2].kind, TokenKind::Star);
        assert_eq!(tokens[3].kind, TokenKind::Slash);
        assert_eq!(tokens[4].kind, TokenKind::EqEq);
        assert_eq!(tokens[5].kind, TokenKind::NotEq);
        assert_eq!(tokens[6].kind, TokenKind::LtEq);
        assert_eq!(tokens[7].kind, TokenKind::GtEq);
        assert_eq!(tokens[8].kind, TokenKind::AmpAmp);
        assert_eq!(tokens[9].kind, TokenKind::PipePipe);
        assert_eq!(tokens[10].kind, TokenKind::Arrow);
        assert_eq!(tokens[11].kind, TokenKind::FatArrow);
    }
}
