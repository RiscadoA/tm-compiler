use crate::annotater::Annot;
use crate::data::{Arm, Exp, Node, Pat};
use std::collections::HashSet;

/// Removes duplicate patterns from match expressions.
pub fn dedup_patterns(ast: Exp<Annot>, changed: &mut bool) -> Exp<Annot> {
    Exp(
        match ast.0 {
            Node::Union { lhs, rhs } => Node::Union {
                lhs: Box::new(dedup_patterns(*lhs, changed)),
                rhs: Box::new(dedup_patterns(*rhs, changed)),
            },

            Node::Match { exp, arms } => {
                let exp = dedup_patterns(*exp, changed);
                let mut new_arms = Vec::new();

                let mut used = HashSet::new();
                let mut set = HashSet::new();

                for arm in arms {
                    let mut pat = match arm.pat {
                        Pat::Union(pat) => Pat::Union(dedup_patterns(pat, changed)),
                        Pat::Any => Pat::Any,
                    };

                    if let Pat::Union(pat) = &mut pat {
                        if pat.union_to_set(&mut set) {
                            let original_sz = set.len();
                            set.retain(|s| !used.contains(s));
                            if set.is_empty() {
                                *changed = true;
                                continue;
                            } else if original_sz != set.len() {
                                *changed = true;
                                *pat = Exp::union_from_set(&set, &pat.1);
                            }

                            for s in set.iter() {
                                used.insert(s.clone());
                            }
                            set.clear();
                        }
                    }

                    new_arms.push(Arm {
                        catch_id: None,
                        pat,
                        exp: dedup_patterns(arm.exp, changed),
                    });
                }

                Node::Match {
                    exp: Box::new(exp),
                    arms: new_arms,
                }
            }

            Node::Function { arg, exp } => Node::Function {
                arg,
                exp: Box::new(dedup_patterns(*exp, changed)),
            },

            Node::Application { func, arg } => Node::Application {
                func: Box::new(dedup_patterns(*func, changed)),
                arg: Box::new(dedup_patterns(*arg, changed)),
            },

            n => n,
        },
        ast.1,
    )
}
