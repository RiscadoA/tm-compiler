use crate::data::{Arm, Exp, Node, Pat};

/// Removes all let bindings from the AST, replacing bindings with function applications.
pub fn remove_lets<Annot>(ast: Exp<Annot>) -> Exp<Annot>
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

            Node::Let { exp, binds } => {
                let mut exp = traverse(*exp);
                for (id, bind) in binds.into_iter().rev() {
                    let annot = bind.1.clone();
                    exp = Exp(
                        Node::Application {
                            func: Box::new(Exp(
                                Node::Function {
                                    arg: id,
                                    exp: Box::new(exp),
                                },
                                annot.clone(),
                            )),
                            arg: Box::new(traverse(bind)),
                        },
                        annot,
                    );
                }
                exp.0
            }

            Node::Function { arg, exp } => Node::Function {
                arg,
                exp: Box::new(traverse(*exp)),
            },

            Node::Application { func, arg } => Node::Application {
                func: Box::new(traverse(*func)),
                arg: Box::new(traverse(*arg)),
            },

            n => n,
        },
        ast.1,
    )
}
