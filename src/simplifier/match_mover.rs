use crate::data::{Arm, Exp, Node, Pat};

/// Moves all matches to top of the function applications so that no match stays in an application.
pub fn move_matches<Annot>(ast: Exp<Annot>) -> Exp<Annot>
where
    Annot: Clone,
{
    traverse(ast)
}

fn traverse<Annot>(ast: Exp<Annot>) -> Exp<Annot>
where
    Annot: Clone,
{
    Exp(
        match ast.0 {
            Node::Union { lhs, rhs } => Node::Union {
                lhs: Box::new(traverse(*lhs)),
                rhs: Box::new(traverse(*rhs)),
            },

            Node::Match { exp, arms } => Node::Match {
                exp: Box::new(traverse(*exp)),
                arms: arms
                    .into_iter()
                    .map(|arm| {
                        let pat = match arm.pat {
                            Pat::Union(exp) => Pat::Union(traverse(exp)),
                            Pat::Any => Pat::Any,
                        };

                        Arm {
                            catch_id: arm.catch_id,
                            pat,
                            exp: traverse(arm.exp),
                        }
                    })
                    .collect(),
            },

            Node::Function { arg, exp } => Node::Function {
                arg,
                exp: Box::new(traverse(*exp)),
            },

            Node::Application { func, arg } => {
                let func = traverse(*func);
                let arg = traverse(*arg);

                if let Node::Match { exp, arms } = func.0 {
                    Node::Match {
                        exp,
                        arms: arms
                            .into_iter()
                            .map(|arm| {
                                let pat = match arm.pat {
                                    Pat::Union(exp) => Pat::Union(traverse(exp)),
                                    Pat::Any => Pat::Any,
                                };

                                Arm {
                                    catch_id: arm.catch_id,
                                    pat,
                                    exp: Exp(
                                        Node::Application {
                                            func: Box::new(traverse(arm.exp)),
                                            arg: Box::new(arg.clone()),
                                        },
                                        ast.1.clone(),
                                    ),
                                }
                            })
                            .collect(),
                    }
                } else if let Node::Match { exp, arms } = arg.0 {
                    Node::Match {
                        exp,
                        arms: arms
                            .into_iter()
                            .map(|arm| {
                                let pat = match arm.pat {
                                    Pat::Union(exp) => Pat::Union(traverse(exp)),
                                    Pat::Any => Pat::Any,
                                };

                                Arm {
                                    catch_id: arm.catch_id,
                                    pat,
                                    exp: Exp(
                                        Node::Application {
                                            func: Box::new(func.clone()),
                                            arg: Box::new(traverse(arm.exp)),
                                        },
                                        ast.1.clone(),
                                    ),
                                }
                            })
                            .collect(),
                    }
                } else {
                    Node::Application {
                        func: Box::new(func),
                        arg: Box::new(arg),
                    }
                }
            }

            n => n,
        },
        ast.1,
    )
}
