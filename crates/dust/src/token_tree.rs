use crate::span::Span;
use crate::token::{
    Token,
    TokenKind,
};

#[derive(Debug, Clone)]
pub enum TokenTree {
    Token(Token),
    Group {
        delimiter: Delimiter,
        open_span: Span,
        close_span: Span,
        children: Vec<TokenTree>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    Paren,   // ()
    Bracket, // []
    Brace,   // {}
}

impl TokenTree {
    pub fn span(&self) -> Span {
        match self {
            TokenTree::Token(t) => t.span,
            TokenTree::Group {
                open_span,
                close_span,
                ..
            } => open_span.merge(*close_span),
        }
    }
}

pub fn build_token_tree(tokens: Vec<Token>) -> Result<Vec<TokenTree>, TreeError> {
    let mut iter = tokens.into_iter().peekable();
    let result = build_tree_inner(&mut iter, None)?;
    Ok(result)
}

fn build_tree_inner(
    iter: &mut std::iter::Peekable<std::vec::IntoIter<Token>>,
    expected_close: Option<(Delimiter, Span)>,
) -> Result<Vec<TokenTree>, TreeError> {
    let mut trees = Vec::new();

    loop {
        let token = match iter.peek() {
            Some(t) => t.clone(),
            None => {
                if let Some((delim, open_span)) = expected_close {
                    return Err(TreeError {
                        pos: open_span.start,
                        msg: format!("unclosed {:?} delimiter", delim),
                    });
                }
                break;
            }
        };

        match &token.kind {
            TokenKind::Eof => {
                if let Some((delim, open_span)) = expected_close {
                    return Err(TreeError {
                        pos: open_span.start,
                        msg: format!("unclosed {:?} delimiter", delim),
                    });
                }
                break;
            }

            TokenKind::LParen | TokenKind::LBracket | TokenKind::LBrace => {
                let open = iter.next().unwrap();
                let delimiter = match open.kind {
                    TokenKind::LParen => Delimiter::Paren,
                    TokenKind::LBracket => Delimiter::Bracket,
                    TokenKind::LBrace => Delimiter::Brace,
                    _ => unreachable!(),
                };
                let children = build_tree_inner(iter, Some((delimiter, open.span)))?;
                let close = iter.next().unwrap();
                trees.push(TokenTree::Group {
                    delimiter,
                    open_span: open.span,
                    close_span: close.span,
                    children,
                });
            }

            TokenKind::RParen => {
                if let Some((Delimiter::Paren, _)) = expected_close {
                    return Ok(trees);
                }
                return Err(TreeError {
                    pos: token.span.start,
                    msg: "unexpected ')'".into(),
                });
            }

            TokenKind::RBracket => {
                if let Some((Delimiter::Bracket, _)) = expected_close {
                    return Ok(trees);
                }
                return Err(TreeError {
                    pos: token.span.start,
                    msg: "unexpected ']'".into(),
                });
            }

            TokenKind::RBrace => {
                if let Some((Delimiter::Brace, _)) = expected_close {
                    return Ok(trees);
                }
                return Err(TreeError {
                    pos: token.span.start,
                    msg: "unexpected '}'".into(),
                });
            }

            _ => {
                trees.push(TokenTree::Token(iter.next().unwrap()));
            }
        }
    }

    Ok(trees)
}

#[derive(Debug)]
pub struct TreeError {
    pub pos: usize,
    pub msg: String,
}

impl std::fmt::Display for TreeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "token tree error at byte {}: {}", self.pos, self.msg)
    }
}

impl std::error::Error for TreeError {}
