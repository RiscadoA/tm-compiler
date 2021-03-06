use crate::annotater::Annot;
use crate::data::{Exp, Node, Pat};

use std::collections::HashSet;

/// If the given expression is a match expression which matches a symbol expression, matches the symbol.
pub fn match_const(ast: Exp<Annot>) -> Exp<Annot> {
    Exp(
        match ast.0 {
            Node::Match { exp, arms } => match exp.0 {
                Node::Symbol(sym) => {
                    let arm = arms.into_iter().find(|arm| {
                        assert!(arm.catch_id.is_none());
                        let pat = match &arm.pat {
                            Pat::Union(pat) => pat,
                            _ => unreachable!(),
                        };

                        let mut set = HashSet::new();
                        assert!(pat.union_to_set(&mut set));

                        set.contains(&sym)
                    });

                    match arm {
                        Some(arm) => return arm.exp,
                        None => Node::Abort,
                    }
                }

                n => Node::Match {
                    exp: Box::new(Exp(n, exp.1)),
                    arms,
                },
            },

            n => n,
        },
        ast.1,
    )
}
