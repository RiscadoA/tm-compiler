use crate::data::{Arm, Exp, Node, Pat};

/// Moves all matches to top of the function applications so that no match stays in an application.
pub fn move_matches<Annot>(ast: Exp<Annot>, changed: &mut bool) -> Exp<Annot>
where
    Annot: Clone,
{
    traverse(ast, changed)
}

fn traverse<Annot>(ast: Exp<Annot>, changed: &mut bool) -> Exp<Annot>
where
    Annot: Clone,
{
    Exp(
        match ast.0 {
            Node::Union { lhs, rhs } => Node::Union {
                lhs: Box::new(traverse(*lhs, changed)),
                rhs: Box::new(traverse(*rhs, changed)),
            },

            Node::Match { exp, arms } => Node::Match {
                exp: Box::new(traverse(*exp, changed)),
                arms: arms
                    .into_iter()
                    .map(|arm| {
                        let pat = match arm.pat {
                            Pat::Union(exp) => Pat::Union(traverse(exp, changed)),
                            Pat::Any => Pat::Any,
                        };

                        Arm {
                            catch_id: arm.catch_id,
                            pat,
                            exp: traverse(arm.exp, changed),
                        }
                    })
                    .collect(),
            },

            Node::Function { arg, exp } => Node::Function {
                arg,
                exp: Box::new(traverse(*exp, changed)),
            },

            Node::Application { func, arg } => {
                let func = traverse(*func, changed);
                let arg = traverse(*arg, changed);

                if let Node::Match { exp, arms } = func.0 {
                    *changed = true;
                    Node::Match {
                        exp,
                        arms: arms
                            .into_iter()
                            .map(|arm| {
                                let pat = match arm.pat {
                                    Pat::Union(exp) => Pat::Union(traverse(exp, changed)),
                                    Pat::Any => Pat::Any,
                                };

                                Arm {
                                    catch_id: arm.catch_id,
                                    pat,
                                    exp: Exp(
                                        Node::Application {
                                            func: Box::new(traverse(arm.exp, changed)),
                                            arg: Box::new(arg.clone()),
                                        },
                                        ast.1.clone(),
                                    ),
                                }
                            })
                            .collect(),
                    }
                } else if let Node::Match { exp, arms } = arg.0 {
                    *changed = true;

                    Node::Match {
                        exp,
                        arms: arms
                            .into_iter()
                            .map(|arm| {
                                let pat = match arm.pat {
                                    Pat::Union(exp) => Pat::Union(traverse(exp, changed)),
                                    Pat::Any => Pat::Any,
                                };

                                Arm {
                                    catch_id: arm.catch_id,
                                    pat,
                                    exp: Exp(
                                        Node::Application {
                                            func: Box::new(func.clone()),
                                            arg: Box::new(traverse(arm.exp, changed)),
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
