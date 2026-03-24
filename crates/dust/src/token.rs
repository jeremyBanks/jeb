use crate::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Literals
    IntLiteral(u64),
    FloatLiteral(f64),
    StringLiteral(String),
    CharLiteral(char),
    BoolLiteral(bool),

    // Identifier
    Ident,

    // Keywords
    Fn,
    Let,
    Mut,
    If,
    Else,
    While,
    Loop,
    Break,
    Continue,
    Return,
    Struct,
    Enum,
    Match,
    Impl,
    Trait,
    Use,
    Mod,
    Pub,
    SelfLower,  // self
    SelfUpper,  // Self
    Super,
    Crate,
    As,
    In,
    For,
    Const,
    Static,
    Type,
    Where,
    Ref,
    Move,
    Async,
    Await,
    Unsafe,
    Extern,
    Dyn,
    Box,
    True,
    False,

    // Delimiters
    LParen,    // (
    RParen,    // )
    LBracket,  // [
    RBracket,  // ]
    LBrace,    // {
    RBrace,    // }

    // Operators
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    Percent,    // %
    Bang,       // !
    Amp,        // &
    Pipe,       // |
    Caret,      // ^
    Shl,        // <<
    Shr,        // >>
    EqEq,       // ==
    NotEq,      // !=
    Lt,         // <
    Gt,         // >
    LtEq,       // <=
    GtEq,       // >=
    AmpAmp,     // &&
    PipePipe,   // ||
    Eq,         // =
    PlusEq,     // +=
    MinusEq,    // -=
    StarEq,     // *=
    SlashEq,    // /=
    PercentEq,  // %=
    AmpEq,      // &=
    PipeEq,     // |=
    CaretEq,    // ^=
    ShlEq,      // <<=
    ShrEq,      // >>=
    DotDot,     // ..
    DotDotEq,   // ..=
    Arrow,      // ->
    FatArrow,   // =>
    ColonColon, // ::
    Dot,        // .

    // Punctuation
    Comma,     // ,
    Semi,      // ;
    Colon,     // :
    Hash,      // #
    Question,  // ?
    Underscore, // _

    // Special
    Eof,
}

impl TokenKind {
    pub fn keyword_from_str(s: &str) -> Option<TokenKind> {
        match s {
            "fn" => Some(TokenKind::Fn),
            "let" => Some(TokenKind::Let),
            "mut" => Some(TokenKind::Mut),
            "if" => Some(TokenKind::If),
            "else" => Some(TokenKind::Else),
            "while" => Some(TokenKind::While),
            "loop" => Some(TokenKind::Loop),
            "break" => Some(TokenKind::Break),
            "continue" => Some(TokenKind::Continue),
            "return" => Some(TokenKind::Return),
            "struct" => Some(TokenKind::Struct),
            "enum" => Some(TokenKind::Enum),
            "match" => Some(TokenKind::Match),
            "impl" => Some(TokenKind::Impl),
            "trait" => Some(TokenKind::Trait),
            "use" => Some(TokenKind::Use),
            "mod" => Some(TokenKind::Mod),
            "pub" => Some(TokenKind::Pub),
            "self" => Some(TokenKind::SelfLower),
            "Self" => Some(TokenKind::SelfUpper),
            "super" => Some(TokenKind::Super),
            "crate" => Some(TokenKind::Crate),
            "as" => Some(TokenKind::As),
            "in" => Some(TokenKind::In),
            "for" => Some(TokenKind::For),
            "true" => Some(TokenKind::True),
            "false" => Some(TokenKind::False),
            "const" => Some(TokenKind::Const),
            "static" => Some(TokenKind::Static),
            "type" => Some(TokenKind::Type),
            "where" => Some(TokenKind::Where),
            "ref" => Some(TokenKind::Ref),
            "move" => Some(TokenKind::Move),
            "async" => Some(TokenKind::Async),
            "await" => Some(TokenKind::Await),
            "unsafe" => Some(TokenKind::Unsafe),
            "extern" => Some(TokenKind::Extern),
            "dyn" => Some(TokenKind::Dyn),
            "Box" => Some(TokenKind::Box),
            _ => None,
        }
    }
}
