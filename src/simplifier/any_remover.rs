use crate::data::{Arm, Exp, Node, Pat};
use std::collections::HashSet;

/// If the given expression is a match expression with 'any' patterns, they are simplified.
pub fn remove_any<Annot>(ast: Exp<Annot>, alphabet: &HashSet<String>) -> Exp<Annot>
where
    Annot: Clone,
{
    Exp(
        match ast.0 {
            Node::Match { exp, arms } => Node::Match {
                exp,
                arms: arms
                    .into_iter()
                    .map(|arm| Arm {
                        pat: match arm.pat {
                            Pat::Any => Pat::Union(Exp::union_from_set(alphabet, &arm.exp.1)),
                            pat => pat,
                        },
                        catch_id: arm.catch_id,
                        exp: arm.exp,
                    })
                    .collect(),
            },

            n => n,
        },
        ast.1,
    )
}
