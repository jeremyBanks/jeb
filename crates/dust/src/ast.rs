use crate::span::Span;
use crate::token::Token;

/// A uniform AST node. All syntactic forms use the same type.
/// Children are labeled for named access by the compiler.
#[derive(Debug, Clone)]
pub struct Node {
    pub kind: &'static str,
    pub span: Span,
    pub children: Vec<(&'static str, Child)>,
}

#[derive(Debug, Clone)]
pub enum Child {
    Node(Box<Node>),
    Token(Token),
    Nodes(Vec<Node>),
}

impl Node {
    pub fn new(kind: &'static str, span: Span) -> Self {
        Self {
            kind,
            span,
            children: Vec::new(),
        }
    }

    pub fn with(mut self, label: &'static str, child: Child) -> Self {
        self.children.push((label, child));
        self
    }

    pub fn child_node(&self, label: &str) -> Option<&Node> {
        for (l, c) in &self.children {
            if *l == label {
                if let Child::Node(n) = c {
                    return Some(n);
                }
            }
        }
        None
    }

    pub fn child_token(&self, label: &str) -> Option<&Token> {
        for (l, c) in &self.children {
            if *l == label {
                if let Child::Token(t) = c {
                    return Some(t);
                }
            }
        }
        None
    }

    pub fn child_nodes(&self, label: &str) -> &[Node] {
        for (l, c) in &self.children {
            if *l == label {
                if let Child::Nodes(ns) = c {
                    return ns;
                }
            }
        }
        &[]
    }

    pub fn has_child(&self, label: &str) -> bool {
        self.children.iter().any(|(l, _)| *l == label)
    }
}
