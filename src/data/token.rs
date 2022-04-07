use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    Symbol(String),

    Match,
    Any,
    Let,
    In,

    LParenthesis,
    RParenthesis,
    LBraces,
    RBraces,
    Colon,
    Arrow,
    Assign,
    Comma,
    Pipe,
    Catch,
}

#[derive(Debug, Clone)]
pub struct TokenLoc {
    line: usize,
    col: usize,
    import: Option<String>,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Identifier(s) => write!(f, "{}", s),
            Token::Symbol(s) => write!(f, "'{}'", s),
            Token::Match => write!(f, "match"),
            Token::Any => write!(f, "any"),
            Token::Let => write!(f, "let"),
            Token::In => write!(f, "in"),
            Token::LParenthesis => write!(f, "("),
            Token::RParenthesis => write!(f, ")"),
            Token::LBraces => write!(f, "{{"),
            Token::RBraces => write!(f, "}}"),
            Token::Colon => write!(f, ":"),
            Token::Arrow => write!(f, ">"),
            Token::Assign => write!(f, "="),
            Token::Comma => write!(f, ","),
            Token::Pipe => write!(f, "|"),
            Token::Catch => write!(f, "@"),
        }
    }
}

impl TokenLoc {
    pub fn new(line: usize, col: usize, import: Option<String>) -> Self {
        Self { line, col, import }
    }
}

impl fmt::Display for TokenLoc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(import) = &self.import {
            write!(
                f,
                "line {}, column {}, import {}",
                self.line, self.col, import
            )
        } else {
            write!(f, "line {}, column {}", self.line, self.col)
        }
    }
}
