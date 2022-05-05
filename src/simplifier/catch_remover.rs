use crate::annotater::Annot;
use crate::data::{Arm, Exp, Node, Pat, Type};
use std::collections::HashSet;

/// Removes match catch variables, replacing the arm expression with a function application.
pub fn remove_catch(ast: Exp<Annot>, changed: &mut bool) -> Exp<Annot> {
    Exp(
        match ast.0 {
            Node::Union { lhs, rhs } => Node::Union {
                lhs: Box::new(remove_catch(*lhs, changed)),
                rhs: Box::new(remove_catch(*rhs, changed)),
            },

            Node::Match { exp, arms } => {
                let exp = remove_catch(*exp, changed);
                let mut new_arms = Vec::new();

                for arm in arms {
                    let exp = remove_catch(arm.exp, changed);

                    if let Some(id) = arm.catch_id {
                        if let Pat::Union(pat) = arm.pat {
                            // Check if the catch pattern is a constant union.
                            // If it isn't, we can't remove the catch.
                            let mut symbols = HashSet::new();
                            if get_union_symbols(&pat, &mut symbols) {
                                // It is a constant union, remove the catch.
                                *changed = true;
                                for sym in symbols {
                                    new_arms.push(Arm {
                                        pat: Pat::Union(Exp(
                                            Node::Symbol(sym.clone()),
                                            pat.1.clone(),
                                        )),
                                        catch_id: None,
                                        exp: Exp(
                                            Node::Application {
                                                func: Box::new(Exp(
                                                    Node::Function {
                                                        arg: id.clone(),
                                                        exp: Box::new(exp.clone()),
                                                    },
                                                    Annot(
                                                        Type::Function {
                                                            arg: Box::new(Type::Symbol),
                                                            ret: Box::new(exp.1 .0.clone()),
                                                        },
                                                        exp.1 .1.clone(),
                                                    ),
                                                )),
                                                arg: Box::new(Exp(
                                                    Node::Symbol(sym.clone()),
                                                    pat.1.clone(),
                                                )),
                                            },
                                            exp.1.clone(),
                                        ),
                                    });
                                }
                            }
                        }
                    } else {
                        new_arms.push(Arm {
                            catch_id: None,
                            pat: arm.pat,
                            exp,
                        });
                    }
                }

                Node::Match {
                    exp: Box::new(exp),
                    arms: new_arms,
                }
            }

            Node::Function { arg, exp } => Node::Function {
                arg,
                exp: Box::new(remove_catch(*exp, changed)),
            },

            Node::Application { func, arg } => Node::Application {
                func: Box::new(remove_catch(*func, changed)),
                arg: Box::new(remove_catch(*arg, changed)),
            },

            n => n,
        },
        ast.1,
    )
}

fn get_union_symbols<Annot>(exp: &Exp<Annot>, symbols: &mut HashSet<String>) -> bool {
    match &exp.0 {
        Node::Symbol(s) => {
            symbols.insert(s.clone());
            true
        }
        Node::Union { lhs, rhs } => {
            get_union_symbols(&lhs, symbols) && get_union_symbols(&rhs, symbols)
        }
        _ => false,
    }
}
