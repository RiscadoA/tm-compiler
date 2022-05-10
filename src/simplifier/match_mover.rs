use crate::data::{Arm, Exp, Node};

/// If the current expression is an application in which either the function or the argument is a match, the match is
/// moved up in the AST so that the expression becomes a match of applications.
pub fn move_matches<Annot>(ast: Exp<Annot>) -> Exp<Annot>
where
    Annot: Clone,
{
    Exp(
        match ast.0 {
            Node::Application { func, arg } => {
                if let Node::Match { exp, arms } = func.0 {
                    Node::Match {
                        exp,
                        arms: arms
                            .into_iter()
                            .map(|arm| Arm {
                                pat: arm.pat,
                                catch_id: arm.catch_id,
                                exp: move_matches(Exp(
                                    Node::Application {
                                        func: Box::new(arm.exp),
                                        arg: arg.clone(),
                                    },
                                    ast.1.clone(),
                                )),
                            })
                            .collect(),
                    }
                } else if let Node::Match { exp, arms } = arg.0 {
                    Node::Match {
                        exp,
                        arms: arms
                            .into_iter()
                            .map(|arm| Arm {
                                pat: arm.pat,
                                catch_id: arm.catch_id,
                                exp: Exp(
                                    Node::Application {
                                        func: func.clone(),
                                        arg: Box::new(arm.exp),
                                    },
                                    ast.1.clone(),
                                ),
                            })
                            .collect(),
                    }
                } else {
                    Node::Application { func, arg }
                }
            }

            n => n,
        },
        ast.1,
    )
}
