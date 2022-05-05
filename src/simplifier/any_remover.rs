use crate::data::{Arm, Exp, Node, Pat};
use std::collections::HashSet;

/// Removes 'any' match patterns, replacing them with the remaining symbols.
pub fn remove_any<Annot>(ast: Exp<Annot>, alphabet: &HashSet<String>) -> Exp<Annot>
where
    Annot: Clone,
{
    Exp(
        match ast.0 {
            Node::Union { lhs, rhs } => Node::Union {
                lhs: Box::new(remove_any(*lhs, alphabet)),
                rhs: Box::new(remove_any(*rhs, alphabet)),
            },

            Node::Match { exp, arms } => {
                let exp = remove_any(*exp, alphabet);
                let mut new_arms = Vec::new();

                for arm in arms {
                    let pat = match arm.pat {
                        Pat::Union(exp) => Pat::Union(remove_any(exp, alphabet)),
                        Pat::Any => Pat::Union(Exp::union_from_set(alphabet, &arm.exp.1)),
                    };

                    new_arms.push(Arm {
                        pat,
                        catch_id: arm.catch_id,
                        exp: remove_any(arm.exp, alphabet),
                    });
                }

                Node::Match {
                    exp: Box::new(exp),
                    arms: new_arms,
                }
            }

            Node::Function { arg, exp } => Node::Function {
                arg,
                exp: Box::new(remove_any(*exp, alphabet)),
            },

            Node::Application { func, arg } => Node::Application {
                func: Box::new(remove_any(*func, alphabet)),
                arg: Box::new(remove_any(*arg, alphabet)),
            },

            n => n,
        },
        ast.1,
    )
}
