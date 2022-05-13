use crate::annotater::Annot;
use crate::data::{Arm, Exp, Node, Pat};
use std::collections::HashSet;

/// Merges all matches which contain matches in their expressions.
pub fn merge_matches(ast: Exp<Annot>) -> Exp<Annot> {
    Exp(
        match ast.0 {
            Node::Match { exp, arms } => {
                if let Node::Match {
                    exp: inner_exp,
                    arms: inner_arms,
                } = exp.0
                {
                    let mut new_arms = Vec::new();

                    for arm in arms {
                        let (pat, annot) = match arm.pat {
                            Pat::Union(u) => {
                                let mut set = HashSet::new();
                                assert!(u.union_to_set(&mut set));
                                (set, u.1)
                            }
                            _ => unreachable!(),
                        };

                        let mut set = HashSet::new();
                        for inner_arm in inner_arms.iter() {
                            match &inner_arm.exp.0 {
                                Node::Symbol(sym) if pat.contains(sym) => match &inner_arm.pat {
                                    Pat::Union(pat) => assert!(pat.union_to_set(&mut set)),
                                    _ => unreachable!(),
                                },
                                Node::Symbol(_) => {}
                                _ => unreachable!(
                                    "\n{}\n{:?}\n{}",
                                    inner_exp, inner_arm.pat, inner_arm.exp
                                ),
                            }
                        }

                        if !set.is_empty() {
                            new_arms.push(Arm {
                                pat: Pat::Union(Exp::union_from_set(&set, &annot)),
                                exp: arm.exp.clone(),
                                catch_id: None,
                            });
                        }
                    }

                    if new_arms.is_empty() {
                        Node::Abort
                    } else {
                        Node::Match {
                            exp: inner_exp,
                            arms: new_arms,
                        }
                    }
                } else {
                    Node::Match { exp, arms }
                }
            }

            n => n,
        },
        ast.1,
    )
}
