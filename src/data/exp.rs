use std::collections::HashSet;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Node<Annot> {
    Identifier(String),
    Symbol(String),
    Abort,

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
        binds: Vec<(String, bool, Exp<Annot>)>,
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

impl<Annot> Exp<Annot> {
    /// Checks if two expressions are equivalent, ignoring annotations.
    pub fn eq_ignore_annot(&self, other: &Exp<Annot>) -> bool {
        match (&self.0, &other.0) {
            (Node::Identifier(id), Node::Identifier(id2)) => id == id2,
            (Node::Symbol(sym), Node::Symbol(sym2)) => sym == sym2,
            (Node::Abort, Node::Abort) => true,
            (
                Node::Union { lhs, rhs },
                Node::Union {
                    lhs: lhs2,
                    rhs: rhs2,
                },
            ) => {
                if lhs.eq_ignore_annot(lhs2) && rhs.eq_ignore_annot(rhs2) {
                    true
                } else {
                    let mut set1 = HashSet::new();
                    let mut set2 = HashSet::new();
                    if lhs.union_to_set(&mut set1)
                        && rhs.union_to_set(&mut set1)
                        && lhs2.union_to_set(&mut set2)
                        && rhs2.union_to_set(&mut set2)
                    {
                        set1 == set2
                    } else {
                        false
                    }
                }
            }
            (
                Node::Match { exp, arms },
                Node::Match {
                    exp: exp2,
                    arms: arms2,
                },
            ) => {
                exp.eq_ignore_annot(exp2)
                    && arms.iter().all(|arm| {
                        arms2
                            .iter()
                            .find(|arm2| {
                                arm.pat.eq_ignore_annot(&arm2.pat)
                                    && arm.catch_id == arm2.catch_id
                                    && arm.exp.eq_ignore_annot(&arm2.exp)
                            })
                            .is_some()
                    })
            }
            (
                Node::Let { exp, binds },
                Node::Let {
                    exp: exp2,
                    binds: binds2,
                },
            ) => {
                exp.eq_ignore_annot(exp2)
                    && binds
                        .iter()
                        .zip(binds2.iter())
                        .all(|(b1, b2)| b1.0 == b2.0 && b1.1 == b1.1 && b1.2.eq_ignore_annot(&b2.2))
            }
            (
                Node::Function { arg, exp },
                Node::Function {
                    arg: arg2,
                    exp: exp2,
                },
            ) => arg == arg2 && exp.eq_ignore_annot(exp2),
            (
                Node::Application { func, arg },
                Node::Application {
                    func: func2,
                    arg: arg2,
                },
            ) => func.eq_ignore_annot(func2) && arg.eq_ignore_annot(arg2),
            _ => false,
        }
    }

    /// Collects every symbol used in the expression, recursively.
    pub fn collect_symbols(&self, set: &mut HashSet<String>) {
        match &self.0 {
            Node::Symbol(s) => {
                set.insert(s.clone());
            }
            Node::Match { exp, arms } => {
                exp.collect_symbols(set);
                for arm in arms {
                    if let Pat::Union(exp) = &arm.pat {
                        exp.collect_symbols(set);
                    }
                    arm.exp.collect_symbols(set);
                }
            }
            Node::Let { exp, binds } => {
                exp.collect_symbols(set);
                for (_, _, exp) in binds {
                    exp.collect_symbols(set);
                }
            }
            Node::Function { exp, .. } => {
                exp.collect_symbols(set);
            }
            Node::Application { func, arg } => {
                func.collect_symbols(set);
                arg.collect_symbols(set);
            }
            _ => (),
        }
    }

    /// Recursively transforms the expression into a new one, using the given function.
    /// The function is first called on every subexpression, and then on the current expression.
    pub fn transform<F>(self, f: &F) -> Exp<Annot>
    where
        F: Fn(Exp<Annot>) -> Exp<Annot>,
    {
        f(Exp(
            match self.0 {
                Node::Union { lhs, rhs } => Node::Union {
                    lhs: Box::new(lhs.transform(f)),
                    rhs: Box::new(rhs.transform(f)),
                },

                Node::Match {
                    exp: match_exp,
                    arms,
                } => Node::Match {
                    exp: Box::new(match_exp.transform(f)),
                    arms: arms
                        .into_iter()
                        .map(|arm| Arm {
                            pat: match arm.pat {
                                Pat::Union(u) => Pat::Union(u.transform(f)),
                                Pat::Any => Pat::Any,
                            },
                            catch_id: arm.catch_id,
                            exp: arm.exp.transform(f),
                        })
                        .collect(),
                },

                Node::Let { exp, binds } => Node::Let {
                    exp: Box::new(exp.transform(f)),
                    binds: binds
                        .into_iter()
                        .map(|(id, optional, exp)| (id, optional, exp.transform(f)))
                        .collect(),
                },

                Node::Function { arg, exp: func_exp } => Node::Function {
                    arg,
                    exp: Box::new(func_exp.transform(f)),
                },

                Node::Application { func, arg } => Node::Application {
                    func: Box::new(func.transform(f)),
                    arg: Box::new(arg.transform(f)),
                },

                n => n,
            },
            self.1,
        ))
    }

    /// Collects symbols used in the expression, if its a union expression.
    /// If its not a union expression or if its not constant, returns false.
    pub fn union_to_set(&self, symbols: &mut HashSet<String>) -> bool {
        match &self.0 {
            Node::Symbol(s) => {
                symbols.insert(s.clone());
                true
            }
            Node::Union { lhs, rhs } => lhs.union_to_set(symbols) && rhs.union_to_set(symbols),
            _ => false,
        }
    }

    /// Generates a union expression from a set of symbols.
    pub fn union_from_set(symbols: &HashSet<String>, annot: &Annot) -> Exp<Annot>
    where
        Annot: Clone,
    {
        let mut symbols = symbols.iter().map(|s| s.clone());
        let mut exp = Exp(Node::Symbol(symbols.next().unwrap()), annot.clone());
        for symbol in symbols {
            exp = Exp(
                Node::Union {
                    lhs: Box::new(exp),
                    rhs: Box::new(Exp(Node::Symbol(symbol), annot.clone())),
                },
                annot.clone(),
            );
        }
        exp
    }
}

impl<Annot> Pat<Annot> {
    pub fn eq_ignore_annot(&self, other: &Pat<Annot>) -> bool {
        match (self, other) {
            (Pat::Union(lhs), Pat::Union(rhs)) => lhs.eq_ignore_annot(rhs),
            (Pat::Any, Pat::Any) => true,
            _ => false,
        }
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
        Node::Abort => writeln!(f, "abort{}", annot),
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
                    writeln!(f, "_ @")?;
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
            for (id, optional, exp) in binds {
                fmt_indent(f, indent + 1)?;
                writeln!(f, "{} {}", id, if *optional { "?" } else { "=" })?;
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
