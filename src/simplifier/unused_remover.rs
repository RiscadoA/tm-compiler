use crate::data::{Exp, Node, Pat};
use std::collections::HashSet;

/// Removes all unused variables from the AST, replacing applications with the function expressions.
pub fn remove_unused<Annot>(ast: Exp<Annot>) -> Exp<Annot>
where
    Annot: Clone,
{
    traverse(ast, &mut HashSet::new())
}

fn traverse<Annot>(ast: Exp<Annot>, used: &mut HashSet<String>) -> Exp<Annot>
where
    Annot: Clone,
{
    Exp(
        match ast.0 {
            Node::Identifier(id) => {
                used.insert(id.clone());
                Node::Identifier(id)
            }

            Node::Union { lhs, rhs } => Node::Union {
                lhs: Box::new(traverse(*lhs, used)),
                rhs: Box::new(traverse(*rhs, used)),
            },

            Node::Match { exp, arms } => Node::Match {
                exp: Box::new(traverse(*exp, used)),
                arms: arms
                    .into_iter()
                    .map(|mut arm| {
                        arm.pat = match arm.pat {
                            Pat::Union(exp) => Pat::Union(traverse(exp, used)),
                            Pat::Any => Pat::Any,
                        };

                        if let Some(id) = arm.catch_id.clone() {
                            let contained = used.contains(&id);
                            used.remove(&id);
                            arm.exp = traverse(arm.exp, used);
                            if !used.contains(&id) {
                                arm.catch_id = None;
                                if contained {
                                    used.insert(id.clone());
                                }
                            } else if !contained {
                                used.remove(&id);
                            }
                        }

                        arm.exp = traverse(arm.exp, used);
                        arm
                    })
                    .collect(),
            },

            Node::Function { arg, exp } => {
                let contained = used.contains(&arg);
                used.remove(&arg);
                let exp = Box::new(traverse(*exp, used));
                if contained {
                    used.insert(arg.clone());
                }
                Node::Function { arg, exp }
            }

            Node::Application { func, arg } => {
                if let Node::Function {
                    arg: arg_id,
                    exp: func_exp,
                } = func.0
                {
                    let contained = used.contains(&arg_id);
                    used.remove(&arg_id);
                    let func_exp = traverse(*func_exp, used);
                    if !used.contains(&arg_id) {
                        if contained {
                            used.insert(arg_id.clone());
                        }
                        return func_exp;
                    } else {
                        if !contained {
                            used.remove(&arg_id);
                        }

                        Node::Application {
                            func: Box::new(Exp(
                                Node::Function {
                                    arg: arg_id,
                                    exp: Box::new(func_exp),
                                },
                                func.1,
                            )),
                            arg: Box::new(traverse(*arg, used)),
                        }
                    }
                } else {
                    Node::Application {
                        func: Box::new(traverse(*func, used)),
                        arg: Box::new(traverse(*arg, used)),
                    }
                }
            }

            n => n,
        },
        ast.1,
    )
}
