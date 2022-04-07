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
