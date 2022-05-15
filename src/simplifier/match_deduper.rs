use crate::annotater::Annot;
use crate::data::{Arm, Exp, Node, Pat, Type};

use super::capture_remover::remove_captures;

/// If the current expression is a match expression which matches a tape, then every match in subexpressions which
/// matches against the same tape is matched to the correct symbol. If there is a set application which uses the same
/// tape, then the set is removed.
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
                                        exp: traverse(
                                            arm.exp,
                                            &id,
                                            &match arm.pat {
                                                Pat::Union(Exp(Node::Symbol(s), _)) => {
                                                    Some(s.clone())
                                                }
                                                _ => None,
                                            },
                                        ),
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

fn traverse(ast: Exp<Annot>, id: &str, sym: &Option<String>) -> Exp<Annot> {
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
                        .filter_map(|arm| {
                            if arm.catch_id != Some(id.to_owned()) {
                                Some(Arm {
                                    pat: match arm.pat {
                                        Pat::Union(pat) => Pat::Union(traverse(pat, id, sym)),
                                        Pat::Any => Pat::Any,
                                    },
                                    catch_id: arm.catch_id,
                                    exp: traverse(arm.exp, id, sym),
                                })
                            } else {
                                None
                            }
                        })
                        .collect(),
                }
            }

            Node::Function { arg, exp } if arg != id => Node::Function {
                arg,
                exp: Box::new(traverse(*exp, id, sym)),
            },

            Node::Application { func, arg } => match (*func, *arg) {
                (func, Exp(Node::Identifier(i), _)) if i == id && is_set(&func, sym) => {
                    return Exp(Node::Identifier(id.to_owned()), ast.1);
                }
                (func, arg) => Node::Application {
                    func: Box::new(traverse(func, id, sym)),
                    arg: Box::new(traverse(arg, id, sym)),
                },
            },

            n => n,
        },
        ast.1,
    )
}

fn is_set(func: &Exp<Annot>, sym: &Option<String>) -> bool {
    let sym = match sym {
        Some(sym) => sym,
        None => return false,
    };

    match &func.0 {
        Node::Application { func, arg } => match (&func.0, &arg.0) {
            (Node::Identifier(id), Node::Symbol(s)) if s == sym && id == "set" => true,
            _ => false,
        },
        _ => false,
    }
}
