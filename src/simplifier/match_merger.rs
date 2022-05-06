use crate::annotater::Annot;
use crate::data::{Arm, Exp, Node, Pat};
use std::collections::HashSet;

/// Merges all matches which contain matches in their expressions.
pub fn merge_matches(ast: Exp<Annot>, changed: &mut bool) -> Result<Exp<Annot>, String> {
    traverse(ast, changed)
}

/// Checks if an expression matches a pattern.
fn matches(
    pat: &Exp<Annot>,
    inner_pat: &Pat<Annot>,
    inner_exp: &Exp<Annot>,
) -> Option<HashSet<String>> {
    let mut set1 = HashSet::new();
    let mut set2 = HashSet::new();
    if pat.union_to_set(&mut set1) && inner_exp.union_to_set(&mut set2) {
        if set1.intersection(&set2).next() == None {
            None
        } else {
            Some(match inner_pat {
                Pat::Union(union) => {
                    let mut set = HashSet::new();
                    union.union_to_set(&mut set);
                    set
                }
                Pat::Any => unreachable!(),
            })
        }
    } else {
        None
    }
}

// Checks if an expression is composed of only unions and symbols.
fn is_simple_union(exp: &Exp<Annot>) -> bool {
    match &exp.0 {
        Node::Union { lhs, rhs } => is_simple_union(&lhs) && is_simple_union(&rhs),
        Node::Symbol(_) => true,
        _ => false,
    }
}

fn traverse(ast: Exp<Annot>, changed: &mut bool) -> Result<Exp<Annot>, String> {
    Ok(Exp(
        match ast.0 {
            Node::Union { lhs, rhs } => Node::Union {
                lhs: Box::new(traverse(*lhs, changed)?),
                rhs: Box::new(traverse(*rhs, changed)?),
            },

            Node::Match { exp, arms } => {
                let mut exp = traverse(*exp, changed)?;
                let mut new_arms = Vec::new();
                let mut merged = false;

                if let Node::Match {
                    exp: inner_exp,
                    arms: inner_arms,
                } = &exp.0
                {
                    // First check if both matches are mergeable right now.
                    // For this, the inner arm expressions and the outer arm patterns must be either symbols or unions of unions / symbols.
                    // Catch IDs should also be removed before this check.
                    merged = true;
                    for arm in arms.iter() {
                        if arm.catch_id != None
                            || !match &arm.pat {
                                Pat::Union(u) => is_simple_union(&u),
                                Pat::Any => false,
                            }
                        {
                            merged = false;
                            break;
                        }
                    }
                    for arm in inner_arms.iter() {
                        if arm.catch_id != None || !is_simple_union(&arm.exp) {
                            merged = false;
                            break;
                        }
                    }

                    if merged {
                        *changed = true;

                        for arm in arms.iter() {
                            let pat = match &arm.pat {
                                Pat::Union(exp) => traverse(exp.clone(), changed)?,
                                Pat::Any => unreachable!(),
                            };

                            let set = inner_arms
                                .iter()
                                .filter_map(|inner_arm| {
                                    matches(&pat, &inner_arm.pat, &inner_arm.exp)
                                })
                                .fold(HashSet::new(), |mut acc, inner_pats| {
                                    acc.extend(inner_pats);
                                    acc
                                });

                            if !set.is_empty() {
                                new_arms.push(Arm {
                                    pat: Pat::Union(Exp::union_from_set(&set, &pat.1)),
                                    exp: arm.exp.clone(),
                                    catch_id: None,
                                });
                            }
                        }

                        exp = *inner_exp.clone();
                    }
                }

                if !merged {
                    for arm in arms {
                        let pat = match arm.pat {
                            Pat::Union(exp) => Pat::Union(traverse(exp, changed)?),
                            Pat::Any => Pat::Any,
                        };

                        new_arms.push(Arm {
                            catch_id: arm.catch_id,
                            pat,
                            exp: traverse(arm.exp, changed)?,
                        })
                    }
                }

                Node::Match {
                    exp: Box::new(exp),
                    arms: new_arms,
                }
            }

            Node::Function { arg, exp } => Node::Function {
                arg,
                exp: Box::new(traverse(*exp, changed)?),
            },

            Node::Application { func, arg } => Node::Application {
                func: Box::new(traverse(*func, changed)?),
                arg: Box::new(traverse(*arg, changed)?),
            },

            n => n,
        },
        ast.1,
    ))
}
