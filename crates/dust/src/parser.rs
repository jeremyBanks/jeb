use crate::ast::{Child, Node};
use crate::span::Span;
use crate::token::TokenKind;
use crate::token_tree::{Delimiter, TokenTree};

pub struct Parser<'a> {
    trees: &'a [TokenTree],
    pos: usize,
}

#[derive(Debug)]
pub struct ParseError {
    pub span: Span,
    pub msg: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "parse error at byte {}: {}", self.span.start, self.msg)
    }
}

impl std::error::Error for ParseError {}

impl<'a> Parser<'a> {
    pub fn new(trees: &'a [TokenTree]) -> Self {
        Self { trees, pos: 0 }
    }

    fn current_span(&self) -> Span {
        if self.pos < self.trees.len() {
            self.trees[self.pos].span()
        } else {
            Span::new(0, 0)
        }
    }

    fn at_end(&self) -> bool {
        self.pos >= self.trees.len()
    }

    fn peek_token_kind(&self) -> Option<&TokenKind> {
        if self.pos < self.trees.len() {
            if let TokenTree::Token(t) = &self.trees[self.pos] {
                return Some(&t.kind);
            }
        }
        None
    }

    fn expect_token(&mut self, expected: &TokenKind) -> Result<crate::token::Token, ParseError> {
        if self.pos >= self.trees.len() {
            return Err(ParseError {
                span: self.current_span(),
                msg: format!("expected {:?}, found end of input", expected),
            });
        }
        if let TokenTree::Token(t) = &self.trees[self.pos] {
            if std::mem::discriminant(&t.kind) == std::mem::discriminant(expected) {
                let token = t.clone();
                self.pos += 1;
                return Ok(token);
            }
            return Err(ParseError {
                span: t.span,
                msg: format!("expected {:?}, found {:?}", expected, t.kind),
            });
        }
        Err(ParseError {
            span: self.current_span(),
            msg: format!("expected {:?}, found group", expected),
        })
    }

    fn eat_token(&mut self, expected: &TokenKind) -> Option<crate::token::Token> {
        if let Some(k) = self.peek_token_kind() {
            if std::mem::discriminant(k) == std::mem::discriminant(expected) {
                if let TokenTree::Token(t) = &self.trees[self.pos] {
                    let token = t.clone();
                    self.pos += 1;
                    return Some(token);
                }
            }
        }
        None
    }

    fn consume_token(&mut self) -> Result<crate::token::Token, ParseError> {
        if self.pos >= self.trees.len() {
            return Err(ParseError {
                span: self.current_span(),
                msg: "unexpected end of input".into(),
            });
        }
        if let TokenTree::Token(t) = &self.trees[self.pos] {
            let token = t.clone();
            self.pos += 1;
            Ok(token)
        } else {
            Err(ParseError {
                span: self.current_span(),
                msg: "expected token, found group".into(),
            })
        }
    }

    fn expect_group(&mut self, delim: Delimiter) -> Result<&'a [TokenTree], ParseError> {
        if self.pos >= self.trees.len() {
            return Err(ParseError {
                span: self.current_span(),
                msg: format!("expected {:?} group, found end of input", delim),
            });
        }
        if let TokenTree::Group {
            delimiter,
            children,
            ..
        } = &self.trees[self.pos]
        {
            if *delimiter == delim {
                self.pos += 1;
                return Ok(children);
            }
            return Err(ParseError {
                span: self.trees[self.pos].span(),
                msg: format!("expected {:?} group, found {:?} group", delim, delimiter),
            });
        }
        Err(ParseError {
            span: self.current_span(),
            msg: format!("expected {:?} group, found token", delim),
        })
    }

    // ── File (top-level items) ──

    pub fn parse_file(&mut self) -> Result<Node, ParseError> {
        let span_start = self.current_span();
        let mut items = Vec::new();

        while !self.at_end() {
            items.push(self.parse_item()?);
        }

        let span = if items.is_empty() {
            span_start
        } else {
            span_start.merge(items.last().unwrap().span)
        };

        Ok(Node::new("file", span).with("items", Child::Nodes(items)))
    }

    // ── Items ──

    fn parse_item(&mut self) -> Result<Node, ParseError> {
        let is_pub = self.eat_token(&TokenKind::Pub).is_some();

        match self.peek_token_kind() {
            Some(TokenKind::Fn) => self.parse_fn(is_pub),
            Some(TokenKind::Struct) => self.parse_struct(is_pub),
            Some(TokenKind::Const) => self.parse_const(is_pub),
            Some(TokenKind::Static) => self.parse_static(is_pub),
            _ => Err(ParseError {
                span: self.current_span(),
                msg: "expected item (fn, struct, const, static)".into(),
            }),
        }
    }

    fn parse_fn(&mut self, _is_pub: bool) -> Result<Node, ParseError> {
        let fn_tok = self.expect_token(&TokenKind::Fn)?;
        let name = self.expect_token(&TokenKind::Ident)?;

        let params_trees = self.expect_group(Delimiter::Paren)?;
        let params = self.parse_fn_params(params_trees)?;

        // Optional return type
        let ret_type = if self.eat_token(&TokenKind::Arrow).is_some() {
            Some(self.parse_type()?)
        } else {
            None
        };

        let body = self.parse_block()?;
        let span = fn_tok.span.merge(body.span);

        let mut node = Node::new("fn", span)
            .with("name", Child::Token(name))
            .with("params", Child::Nodes(params))
            .with("body", Child::Node(Box::new(body)));

        if let Some(rt) = ret_type {
            node = node.with("return_type", Child::Node(Box::new(rt)));
        }

        Ok(node)
    }

    fn parse_fn_params(&self, trees: &[TokenTree]) -> Result<Vec<Node>, ParseError> {
        let mut params = Vec::new();
        let mut sub = Parser::new(trees);

        while !sub.at_end() {
            let name = sub.expect_token(&TokenKind::Ident)?;
            sub.expect_token(&TokenKind::Colon)?;
            let ty = sub.parse_type()?;
            let span = name.span.merge(ty.span);
            params.push(
                Node::new("param", span)
                    .with("name", Child::Token(name))
                    .with("type", Child::Node(Box::new(ty))),
            );
            sub.eat_token(&TokenKind::Comma);
        }

        Ok(params)
    }

    fn parse_struct(&mut self, _is_pub: bool) -> Result<Node, ParseError> {
        let struct_tok = self.expect_token(&TokenKind::Struct)?;
        let name = self.expect_token(&TokenKind::Ident)?;
        let fields_trees = self.expect_group(Delimiter::Brace)?;
        let fields = self.parse_struct_fields(fields_trees)?;
        let span = struct_tok.span.merge(self.current_span());

        Ok(Node::new("struct", span)
            .with("name", Child::Token(name))
            .with("fields", Child::Nodes(fields)))
    }

    fn parse_struct_fields(&self, trees: &[TokenTree]) -> Result<Vec<Node>, ParseError> {
        let mut fields = Vec::new();
        let mut sub = Parser::new(trees);

        while !sub.at_end() {
            let _is_pub = sub.eat_token(&TokenKind::Pub).is_some();
            let name = sub.expect_token(&TokenKind::Ident)?;
            sub.expect_token(&TokenKind::Colon)?;
            let ty = sub.parse_type()?;
            let span = name.span.merge(ty.span);
            fields.push(
                Node::new("field", span)
                    .with("name", Child::Token(name))
                    .with("type", Child::Node(Box::new(ty))),
            );
            sub.eat_token(&TokenKind::Comma);
        }

        Ok(fields)
    }

    fn parse_const(&mut self, _is_pub: bool) -> Result<Node, ParseError> {
        let const_tok = self.expect_token(&TokenKind::Const)?;
        let name = self.expect_token(&TokenKind::Ident)?;
        self.expect_token(&TokenKind::Colon)?;
        let ty = self.parse_type()?;
        self.expect_token(&TokenKind::Eq)?;
        let value = self.parse_expr()?;
        self.expect_token(&TokenKind::Semi)?;
        let span = const_tok.span.merge(value.span);
        Ok(Node::new("const", span)
            .with("name", Child::Token(name))
            .with("type", Child::Node(Box::new(ty)))
            .with("value", Child::Node(Box::new(value))))
    }

    fn parse_static(&mut self, _is_pub: bool) -> Result<Node, ParseError> {
        let static_tok = self.expect_token(&TokenKind::Static)?;
        let _is_mut = self.eat_token(&TokenKind::Mut).is_some();
        let name = self.expect_token(&TokenKind::Ident)?;
        self.expect_token(&TokenKind::Colon)?;
        let ty = self.parse_type()?;
        self.expect_token(&TokenKind::Eq)?;
        let value = self.parse_expr()?;
        self.expect_token(&TokenKind::Semi)?;
        let span = static_tok.span.merge(value.span);
        Ok(Node::new("static", span)
            .with("name", Child::Token(name))
            .with("type", Child::Node(Box::new(ty)))
            .with("value", Child::Node(Box::new(value))))
    }

    // ── Types ──

    fn parse_type(&mut self) -> Result<Node, ParseError> {
        // Simple named type for now (i32, f64, bool, etc.)
        if let Some(TokenKind::Amp) = self.peek_token_kind() {
            let amp = self.consume_token()?;
            let _is_mut = self.eat_token(&TokenKind::Mut).is_some();
            let inner = self.parse_type()?;
            let span = amp.span.merge(inner.span);
            return Ok(Node::new("ref_type", span).with("inner", Child::Node(Box::new(inner))));
        }

        if let Some(TokenKind::LBracket) = self.peek_token_kind() {
            // Check if this is a group [ ... ]
            if let Some(TokenTree::Group { delimiter: Delimiter::Bracket, .. }) = self.trees.get(self.pos) {
                let trees = self.expect_group(Delimiter::Bracket)?;
                let mut sub = Parser::new(trees);
                let elem_type = sub.parse_type()?;
                let span = elem_type.span;
                return Ok(Node::new("slice_type", span)
                    .with("elem", Child::Node(Box::new(elem_type))));
            }
        }

        let name = self.expect_token(&TokenKind::Ident)?;
        let span = name.span;
        Ok(Node::new("named_type", span).with("name", Child::Token(name)))
    }

    // ── Block ──

    fn parse_block(&mut self) -> Result<Node, ParseError> {
        let block_trees = self.expect_group(Delimiter::Brace)?;
        let mut sub = Parser::new(block_trees);
        let mut stmts = Vec::new();

        let span_start = sub.current_span();

        while !sub.at_end() {
            stmts.push(sub.parse_stmt()?);
        }

        let span = if stmts.is_empty() {
            span_start
        } else {
            span_start.merge(stmts.last().unwrap().span)
        };

        Ok(Node::new("block", span).with("stmts", Child::Nodes(stmts)))
    }

    // ── Statements ──

    fn parse_stmt(&mut self) -> Result<Node, ParseError> {
        match self.peek_token_kind() {
            Some(TokenKind::Let) => self.parse_let(),
            Some(TokenKind::Return) => self.parse_return(),
            Some(TokenKind::While) => self.parse_while(),
            Some(TokenKind::Loop) => self.parse_loop(),
            Some(TokenKind::Break) => {
                let tok = self.consume_token()?;
                self.eat_token(&TokenKind::Semi);
                Ok(Node::new("break", tok.span))
            }
            Some(TokenKind::Continue) => {
                let tok = self.consume_token()?;
                self.eat_token(&TokenKind::Semi);
                Ok(Node::new("continue", tok.span))
            }
            Some(TokenKind::If) => self.parse_if(),
            _ => {
                let expr = self.parse_expr()?;

                // Check for assignment
                if let Some(op_kind) = self.peek_token_kind().cloned() {
                    if matches!(
                        op_kind,
                        TokenKind::Eq
                            | TokenKind::PlusEq
                            | TokenKind::MinusEq
                            | TokenKind::StarEq
                            | TokenKind::SlashEq
                    ) {
                        let op = self.consume_token()?;
                        let rhs = self.parse_expr()?;
                        self.expect_token(&TokenKind::Semi)?;
                        let span = expr.span.merge(rhs.span);
                        return Ok(Node::new("assign", span)
                            .with("target", Child::Node(Box::new(expr)))
                            .with("op", Child::Token(op))
                            .with("value", Child::Node(Box::new(rhs))));
                    }
                }

                // Expression statement
                let ate_semi = self.eat_token(&TokenKind::Semi).is_some();
                let span = expr.span;
                if ate_semi {
                    Ok(Node::new("expr_stmt", span).with("expr", Child::Node(Box::new(expr))))
                } else {
                    // Trailing expression (implicit return in block)
                    Ok(Node::new("tail_expr", span).with("expr", Child::Node(Box::new(expr))))
                }
            }
        }
    }

    fn parse_let(&mut self) -> Result<Node, ParseError> {
        let let_tok = self.expect_token(&TokenKind::Let)?;
        let _is_mut = self.eat_token(&TokenKind::Mut).is_some();
        let name = self.expect_token(&TokenKind::Ident)?;

        let ty = if self.eat_token(&TokenKind::Colon).is_some() {
            Some(self.parse_type()?)
        } else {
            None
        };

        let init = if self.eat_token(&TokenKind::Eq).is_some() {
            Some(self.parse_expr()?)
        } else {
            None
        };

        self.expect_token(&TokenKind::Semi)?;

        let span = let_tok.span.merge(self.current_span());
        let mut node = Node::new("let", span).with("name", Child::Token(name));
        if let Some(t) = ty {
            node = node.with("type", Child::Node(Box::new(t)));
        }
        if let Some(i) = init {
            node = node.with("init", Child::Node(Box::new(i)));
        }
        Ok(node)
    }

    fn parse_return(&mut self) -> Result<Node, ParseError> {
        let ret_tok = self.expect_token(&TokenKind::Return)?;
        if self.eat_token(&TokenKind::Semi).is_some() {
            return Ok(Node::new("return", ret_tok.span));
        }
        let value = self.parse_expr()?;
        self.expect_token(&TokenKind::Semi)?;
        let span = ret_tok.span.merge(value.span);
        Ok(Node::new("return", span).with("value", Child::Node(Box::new(value))))
    }

    fn parse_while(&mut self) -> Result<Node, ParseError> {
        let while_tok = self.expect_token(&TokenKind::While)?;
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        let span = while_tok.span.merge(body.span);
        Ok(Node::new("while", span)
            .with("cond", Child::Node(Box::new(cond)))
            .with("body", Child::Node(Box::new(body))))
    }

    fn parse_loop(&mut self) -> Result<Node, ParseError> {
        let loop_tok = self.expect_token(&TokenKind::Loop)?;
        let body = self.parse_block()?;
        let span = loop_tok.span.merge(body.span);
        Ok(Node::new("loop", span).with("body", Child::Node(Box::new(body))))
    }

    fn parse_if(&mut self) -> Result<Node, ParseError> {
        let if_tok = self.expect_token(&TokenKind::If)?;
        let cond = self.parse_expr()?;
        let then_block = self.parse_block()?;
        let mut span = if_tok.span.merge(then_block.span);

        let mut node = Node::new("if", span)
            .with("cond", Child::Node(Box::new(cond)))
            .with("then", Child::Node(Box::new(then_block)));

        if self.eat_token(&TokenKind::Else).is_some() {
            if let Some(TokenKind::If) = self.peek_token_kind() {
                let else_if = self.parse_if()?;
                span = span.merge(else_if.span);
                node.span = span;
                node = node.with("else", Child::Node(Box::new(else_if)));
            } else {
                let else_block = self.parse_block()?;
                span = span.merge(else_block.span);
                node.span = span;
                node = node.with("else", Child::Node(Box::new(else_block)));
            }
        }

        Ok(node)
    }

    // ── Expressions (Pratt parser) ──

    fn parse_expr(&mut self) -> Result<Node, ParseError> {
        self.parse_expr_bp(0)
    }

    fn parse_expr_bp(&mut self, min_bp: u8) -> Result<Node, ParseError> {
        let mut lhs = self.parse_prefix()?;

        loop {
            if self.at_end() {
                break;
            }

            // Postfix: function call
            if let Some(TokenTree::Group {
                delimiter: Delimiter::Paren,
                ..
            }) = self.trees.get(self.pos)
            {
                let args_trees = self.expect_group(Delimiter::Paren)?;
                let args = self.parse_call_args(args_trees)?;
                let span = lhs.span.merge(self.current_span());
                lhs = Node::new("call", span)
                    .with("callee", Child::Node(Box::new(lhs)))
                    .with("args", Child::Nodes(args));
                continue;
            }

            // Postfix: index
            if let Some(TokenTree::Group {
                delimiter: Delimiter::Bracket,
                ..
            }) = self.trees.get(self.pos)
            {
                let idx_trees = self.expect_group(Delimiter::Bracket)?;
                let mut sub = Parser::new(idx_trees);
                let index = sub.parse_expr()?;
                let span = lhs.span.merge(self.current_span());
                lhs = Node::new("index", span)
                    .with("object", Child::Node(Box::new(lhs)))
                    .with("index", Child::Node(Box::new(index)));
                continue;
            }

            // Field access
            if let Some(TokenKind::Dot) = self.peek_token_kind() {
                self.consume_token()?;
                let field = self.expect_token(&TokenKind::Ident)?;
                let span = lhs.span.merge(field.span);
                lhs = Node::new("field_access", span)
                    .with("object", Child::Node(Box::new(lhs)))
                    .with("field", Child::Token(field));
                continue;
            }

            // Binary operators
            let Some(kind) = self.peek_token_kind().cloned() else {
                break;
            };
            let Some((l_bp, r_bp)) = infix_binding_power(&kind) else {
                break;
            };
            if l_bp < min_bp {
                break;
            }

            let op = self.consume_token()?;
            let rhs = self.parse_expr_bp(r_bp)?;
            let span = lhs.span.merge(rhs.span);
            lhs = Node::new("binary", span)
                .with("lhs", Child::Node(Box::new(lhs)))
                .with("op", Child::Token(op))
                .with("rhs", Child::Node(Box::new(rhs)));
        }

        Ok(lhs)
    }

    fn parse_prefix(&mut self) -> Result<Node, ParseError> {
        match self.peek_token_kind().cloned() {
            Some(TokenKind::Minus) | Some(TokenKind::Bang) => {
                let op = self.consume_token()?;
                let operand = self.parse_expr_bp(prefix_bp())?;
                let span = op.span.merge(operand.span);
                Ok(Node::new("unary", span)
                    .with("op", Child::Token(op))
                    .with("operand", Child::Node(Box::new(operand))))
            }
            Some(TokenKind::Amp) => {
                let op = self.consume_token()?;
                let _is_mut = self.eat_token(&TokenKind::Mut).is_some();
                let operand = self.parse_expr_bp(prefix_bp())?;
                let span = op.span.merge(operand.span);
                Ok(Node::new("ref", span)
                    .with("op", Child::Token(op))
                    .with("operand", Child::Node(Box::new(operand))))
            }
            Some(TokenKind::Star) => {
                let op = self.consume_token()?;
                let operand = self.parse_expr_bp(prefix_bp())?;
                let span = op.span.merge(operand.span);
                Ok(Node::new("deref", span)
                    .with("op", Child::Token(op))
                    .with("operand", Child::Node(Box::new(operand))))
            }
            _ => self.parse_atom(),
        }
    }

    fn parse_atom(&mut self) -> Result<Node, ParseError> {
        if self.at_end() {
            return Err(ParseError {
                span: self.current_span(),
                msg: "expected expression".into(),
            });
        }

        // Parenthesized expression
        if let Some(TokenTree::Group {
            delimiter: Delimiter::Paren,
            ..
        }) = self.trees.get(self.pos)
        {
            let inner = self.expect_group(Delimiter::Paren)?;
            let mut sub = Parser::new(inner);
            return sub.parse_expr();
        }

        // Block expression
        if let Some(TokenTree::Group {
            delimiter: Delimiter::Brace,
            ..
        }) = self.trees.get(self.pos)
        {
            return self.parse_block();
        }

        // If expression
        if let Some(TokenKind::If) = self.peek_token_kind() {
            return self.parse_if();
        }

        match self.peek_token_kind().cloned() {
            Some(TokenKind::IntLiteral(_))
            | Some(TokenKind::FloatLiteral(_))
            | Some(TokenKind::StringLiteral(_))
            | Some(TokenKind::CharLiteral(_))
            | Some(TokenKind::True)
            | Some(TokenKind::False) => {
                let tok = self.consume_token()?;
                let span = tok.span;
                Ok(Node::new("literal", span).with("value", Child::Token(tok)))
            }

            Some(TokenKind::Ident) => {
                let tok = self.consume_token()?;

                // Check for path separator ::
                if let Some(TokenKind::ColonColon) = self.peek_token_kind() {
                    // For now, just consume a simple A::B path
                    let mut path_tokens = vec![tok.clone()];
                    while self.eat_token(&TokenKind::ColonColon).is_some() {
                        let segment = self.expect_token(&TokenKind::Ident)?;
                        path_tokens.push(segment);
                    }
                    let last = path_tokens.last().unwrap();
                    let span = tok.span.merge(last.span);
                    return Ok(Node::new("path", span).with("root", Child::Token(tok)));
                }

                let span = tok.span;
                Ok(Node::new("ident", span).with("name", Child::Token(tok)))
            }

            _ => Err(ParseError {
                span: self.current_span(),
                msg: format!(
                    "unexpected token in expression: {:?}",
                    self.peek_token_kind()
                ),
            }),
        }
    }

    fn parse_call_args(&self, trees: &[TokenTree]) -> Result<Vec<Node>, ParseError> {
        let mut args = Vec::new();
        let mut sub = Parser::new(trees);
        while !sub.at_end() {
            args.push(sub.parse_expr()?);
            sub.eat_token(&TokenKind::Comma);
        }
        Ok(args)
    }
}

fn infix_binding_power(kind: &TokenKind) -> Option<(u8, u8)> {
    match kind {
        TokenKind::PipePipe => Some((1, 2)),
        TokenKind::AmpAmp => Some((3, 4)),
        TokenKind::EqEq | TokenKind::NotEq => Some((5, 6)),
        TokenKind::Lt | TokenKind::Gt | TokenKind::LtEq | TokenKind::GtEq => Some((7, 8)),
        TokenKind::Pipe => Some((9, 10)),
        TokenKind::Caret => Some((11, 12)),
        TokenKind::Amp => Some((13, 14)),
        TokenKind::Shl | TokenKind::Shr => Some((15, 16)),
        TokenKind::Plus | TokenKind::Minus => Some((17, 18)),
        TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Some((19, 20)),
        TokenKind::As => Some((21, 22)),
        _ => None,
    }
}

fn prefix_bp() -> u8 {
    23
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::token_tree::build_token_tree;

    fn parse_str(s: &str) -> Node {
        let mut lexer = Lexer::new(s);
        let tokens = lexer.tokenize().unwrap();
        let trees = build_token_tree(tokens).unwrap();
        Parser::new(&trees).parse_file().unwrap()
    }

    #[test]
    fn test_empty_fn() {
        let ast = parse_str("fn main() {}");
        assert_eq!(ast.kind, "file");
        let items = ast.child_nodes("items");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].kind, "fn");
        assert_eq!(items[0].child_token("name").unwrap().text, "main");
    }

    #[test]
    fn test_fn_with_params() {
        let ast = parse_str("fn add(a: i32, b: i32) -> i32 { return a; }");
        let items = ast.child_nodes("items");
        let func = &items[0];
        assert_eq!(func.child_nodes("params").len(), 2);
        assert!(func.has_child("return_type"));
    }

    #[test]
    fn test_let_stmt() {
        let ast = parse_str("fn f() { let x: i32 = 42; }");
        let func = &ast.child_nodes("items")[0];
        let body = func.child_node("body").unwrap();
        let stmts = body.child_nodes("stmts");
        assert_eq!(stmts.len(), 1);
        assert_eq!(stmts[0].kind, "let");
    }

    #[test]
    fn test_binary_expr() {
        let ast = parse_str("fn f() { let x: i32 = 1 + 2 * 3; }");
        let func = &ast.child_nodes("items")[0];
        let body = func.child_node("body").unwrap();
        let stmts = body.child_nodes("stmts");
        let init = stmts[0].child_node("init").unwrap();
        // Should be (1 + (2 * 3)) due to precedence
        assert_eq!(init.kind, "binary");
        let rhs = init.child_node("rhs").unwrap();
        assert_eq!(rhs.kind, "binary");
    }

    #[test]
    fn test_if_else() {
        let ast = parse_str("fn f() { if x { 1; } else { 2; } }");
        let func = &ast.child_nodes("items")[0];
        let body = func.child_node("body").unwrap();
        let stmts = body.child_nodes("stmts");
        assert_eq!(stmts[0].kind, "if");
        assert!(stmts[0].has_child("else"));
    }
}
