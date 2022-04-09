use std::fmt;

use super::token::TokenLoc;

#[derive(Debug, Clone, PartialEq)]
pub enum Node<Annot> {
    Identifier(String),
    Symbol(String),

    Union {
        lhs: Box<Exp<Annot>>,
        rhs: Box<Exp<Annot>>,
    },

    Match {
        exp: Box<Exp<Annot>>,
        arms: Vec<Arm<Annot>>,
    },

    Let {
        exp: Box<Exp<Annot>>,
        binds: Vec<(String, Exp<Annot>)>,
    },

    Function {
        arg: String,
        exp: Box<Exp<Annot>>,
    },

    Application {
        func: Box<Exp<Annot>>,
        arg: Box<Exp<Annot>>,
    },
}

/// Represents a match arm.
#[derive(Debug, Clone, PartialEq)]
pub struct Arm<Annot> {
    pub catch_id: Option<String>,
    pub pat: Pat<Annot>,
    pub exp: Exp<Annot>,
}

/// Represents a match pattern.
#[derive(Debug, Clone, PartialEq)]
pub enum Pat<Annot> {
    Union(Exp<Annot>),
    Any,
}

/// Represents an expression in the abstract syntax tree.
#[derive(Debug, Clone, PartialEq)]
pub struct Exp<Annot>(pub Node<Annot>, pub Annot);

pub trait AnnotLoc {
    fn loc(&self) -> &TokenLoc;
}

impl AnnotLoc for TokenLoc {
    fn loc(&self) -> &TokenLoc {
        self
    }
}

impl<Annot> fmt::Display for Exp<Annot>
where
    Annot: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt_expression(f, self, 0)
    }
}

fn fmt_expression<Annot>(f: &mut fmt::Formatter, exp: &Exp<Annot>, indent: usize) -> fmt::Result
where
    Annot: fmt::Display,
{
    let fmt_indent = |f: &mut fmt::Formatter, indent| write!(f, "{}", ". ".repeat(indent));
    let annot = if f.alternate() {
        format!(" ({})", exp.1)
    } else {
        "".to_owned()
    };

    fmt_indent(f, indent)?;
    match &exp.0 {
        Node::Identifier(id) => writeln!(f, "{}{}", id, annot),
        Node::Symbol(sym) => writeln!(f, "'{}'{}", sym, annot),
        Node::Union { lhs, rhs } => {
            writeln!(f, "|{}", annot)?;
            fmt_expression(f, &lhs, indent + 1)?;
            fmt_expression(f, &rhs, indent + 1)
        }
        Node::Match { exp: mexp, arms } => {
            writeln!(f, "match{}", annot)?;
            fmt_expression(f, &mexp, indent + 1)?;
            for arm in arms.iter() {
                fmt_indent(f, indent + 1)?;
                if let Some(id) = &arm.catch_id {
                    writeln!(f, "{} @", id)?;
                } else {
                    writeln!(f, "! @")?;
                }

                match &arm.pat {
                    Pat::Union(pat) => fmt_expression(f, &pat, indent + 2)?,
                    Pat::Any => {
                        fmt_indent(f, indent + 2)?;
                        writeln!(f, "any")?;
                    }
                }
                fmt_expression(f, &arm.exp, indent + 2)?;
            }
            Ok(())
        }
        Node::Let { exp: body, binds } => {
            writeln!(f, "let{}", annot)?;
            for (id, exp) in binds {
                fmt_indent(f, indent + 1)?;
                writeln!(f, "{} =", id)?;
                fmt_expression(f, &exp, indent + 2)?;
            }

            fmt_indent(f, indent)?;
            writeln!(f, "in")?;
            fmt_expression(f, &body, indent + 1)
        }
        Node::Function { arg, exp: body } => {
            writeln!(f, "{}:{}", arg, annot)?;
            fmt_expression(f, &body, indent + 1)
        }
        Node::Application { func, arg } => {
            writeln!(f, "${}", annot)?;
            fmt_expression(f, &func, indent + 1)?;
            fmt_expression(f, &arg, indent + 1)
        }
    }
}
