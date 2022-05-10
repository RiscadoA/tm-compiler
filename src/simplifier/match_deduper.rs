use crate::annotater::Annot;
use crate::data::{Arm, Exp, Node, Pat, Type};

use super::capture_remover::remove_captures;

/// If the current expression is a match expression which matches a tape, then every match in subexpressions which
/// matches against the same tape is matched to the correct symbol.
pub fn dedup_matches<F>(ast: Exp<Annot>, rec: F) -> Exp<Annot>
where
    F: Fn(Exp<Annot>) -> Exp<Annot>,
{
    Exp(
        match ast.0 {
            Node::Match { exp, arms } => match *exp {
                Exp(Node::Identifier(id), Annot(Type::Tape, loc)) => {
                    return remove_captures(
                        Exp(
                            Node::Match {
                                exp: Box::new(Exp(
                                    Node::Identifier(id.clone()),
                                    Annot(Type::Tape, loc),
                                )),
                                arms: arms
                                    .into_iter()
                                    .map(|arm| Arm {
                                        pat: arm.pat.clone(),
                                        catch_id: Some("get".to_owned()),
                                        exp: traverse(arm.exp, &id),
                                    })
                                    .collect(),
                            },
                            ast.1,
                        ),
                        rec,
                    );
                }
                _ => Node::Match { exp, arms },
            },

            n => n,
        },
        ast.1,
    )
}

fn traverse(ast: Exp<Annot>, id: &str) -> Exp<Annot> {
    Exp(
        match ast.0 {
            Node::Match { exp, arms } => {
                let (exp, arms) = match exp.0 {
                    Node::Identifier(i) if i == id => (
                        Box::new(Exp(Node::Identifier("get".to_owned()), exp.1)),
                        arms,
                    ),
                    _ => (exp, arms),
                };

                Node::Match {
                    exp,
                    arms: arms
                        .into_iter()
                        .map(|arm| Arm {
                            pat: match arm.pat {
                                Pat::Union(pat) => Pat::Union(traverse(pat, id)),
                                Pat::Any => Pat::Any,
                            },
                            catch_id: arm.catch_id,
                            exp: traverse(arm.exp, id),
                        })
                        .collect(),
                }
            }

            Node::Function { arg, exp } if arg != id => Node::Function {
                arg,
                exp: Box::new(traverse(*exp, id)),
            },

            Node::Application { func, arg } => Node::Application {
                func: Box::new(traverse(*func, id)),
                arg: Box::new(traverse(*arg, id)),
            },

            n => n,
        },
        ast.1,
    )
}
