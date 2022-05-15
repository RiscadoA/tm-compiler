use crate::data::{Arm, Exp, Node, Pat};

/// Replaces all occurences of the given identifier with the given expression.
pub fn replace_id<Annot>(ast: Exp<Annot>, id: &str, exp: &Exp<Annot>) -> Exp<Annot>
where
    Annot: Clone,
{
    Exp(
        match ast.0 {
            Node::Identifier(id2) if id == id2 => return exp.clone(),

            Node::Union { lhs, rhs } => Node::Union {
                lhs: Box::new(replace_id(*lhs, id, exp)),
                rhs: Box::new(replace_id(*rhs, id, exp)),
            },

            Node::Match {
                exp: match_exp,
                arms,
            } => Node::Match {
                exp: Box::new(replace_id(*match_exp, id, exp)),
                arms: arms
                    .into_iter()
                    .map(|arm| {
                        let arm_exp = if arm.catch_id == Some(id.to_owned()) {
                            arm.exp
                        } else {
                            replace_id(arm.exp, id, exp)
                        };

                        Arm {
                            pat: match arm.pat {
                                Pat::Union(u) => Pat::Union(replace_id(u, id, exp)),
                                _ => unreachable!(),
                            },
                            catch_id: arm.catch_id,
                            exp: arm_exp,
                        }
                    })
                    .collect(),
            },

            Node::Function { arg, exp: func_exp } if arg != id => Node::Function {
                arg,
                exp: Box::new(replace_id(*func_exp, id, exp)),
            },

            Node::Application { func, arg } => Node::Application {
                func: Box::new(replace_id(*func, id, exp)),
                arg: Box::new(replace_id(*arg, id, exp)),
            },

            n => n,
        },
        ast.1,
    )
}
