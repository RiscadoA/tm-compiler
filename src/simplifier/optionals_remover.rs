use crate::data::{Arm, Exp, Node, Pat};
use std::collections::HashSet;

/// If the given expression is a let expression, it is simplified into function applications.
pub fn remove_optionals<Annot>(ast: Exp<Annot>) -> Exp<Annot>
where
    Annot: Clone,
{
    traverse(ast, &HashSet::new())
}

fn traverse<Annot>(ast: Exp<Annot>, env: &HashSet<String>) -> Exp<Annot>
where
    Annot: Clone,
{
    Exp(
        match ast.0 {
            Node::Union { lhs, rhs } => Node::Union {
                lhs: Box::new(traverse(*lhs, env)),
                rhs: Box::new(traverse(*rhs, env)),
            },

            Node::Match {
                exp: match_exp,
                arms,
            } => Node::Match {
                exp: Box::new(traverse(*match_exp, env)),
                arms: arms
                    .into_iter()
                    .map(|arm| {
                        let arm_exp = if let Some(id) = &arm.catch_id {
                            let mut env = env.clone();
                            env.insert(id.clone());
                            traverse(arm.exp, &env)
                        } else {
                            traverse(arm.exp, env)
                        };

                        Arm {
                            pat: match arm.pat {
                                Pat::Union(u) => Pat::Union(traverse(u, env)),
                                Pat::Any => Pat::Any,
                            },
                            catch_id: arm.catch_id,
                            exp: arm_exp,
                        }
                    })
                    .collect(),
            },

            Node::Let {
                exp: let_exp,
                binds,
            } => {
                let mut env = env.clone();
                let mut new_binds = Vec::new();

                for bind in binds {
                    if bind.1 && env.contains(&bind.0) {
                        continue;
                    }

                    new_binds.push((bind.0.clone(), false, traverse(bind.2, &env)));
                    env.insert(bind.0.clone());
                }

                Node::Let {
                    exp: Box::new(traverse(*let_exp, &env)),
                    binds: new_binds,
                }
            }

            Node::Function { arg, exp: func_exp } => {
                let mut env = env.clone();
                env.insert(arg.clone());
                Node::Function {
                    arg,
                    exp: Box::new(traverse(*func_exp, &env)),
                }
            }

            Node::Application { func, arg } => Node::Application {
                func: Box::new(traverse(*func, env)),
                arg: Box::new(traverse(*arg, env)),
            },

            n => n,
        },
        ast.1,
    )
}
