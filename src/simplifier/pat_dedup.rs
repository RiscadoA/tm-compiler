use crate::annotater::Annot;
use crate::data::{Arm, Exp, Node, Pat};
use std::collections::HashSet;

/// If the given expression is a match expression, any duplicate patterns are removed.
pub fn dedup_patterns(ast: Exp<Annot>) -> Exp<Annot> {
    Exp(
        match ast.0 {
            Node::Match { exp, arms } => {
                let mut new_arms = Vec::new();

                let mut used = HashSet::new();
                let mut set = HashSet::new();

                for arm in arms {
                    let annot = match arm.pat {
                        Pat::Union(pat) => {
                            assert!(pat.union_to_set(&mut set));
                            pat.1
                        }
                        _ => unreachable!(),
                    };

                    set.retain(|s| !used.contains(s));
                    if set.is_empty() {
                        continue;
                    }

                    new_arms.push(Arm {
                        pat: Pat::Union(Exp::union_from_set(&set, &annot)),
                        catch_id: None,
                        exp: arm.exp,
                    });

                    used.extend(set.drain());
                }

                Node::Match {
                    exp,
                    arms: new_arms,
                }
            }

            n => n,
        },
        ast.1,
    )
}
