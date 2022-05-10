use super::id_replacer::replace_id;
use crate::annotater::Annot;
use crate::data::{Arm, Exp, Node, Pat, Type};
use std::collections::HashSet;

/// Removes match capture variables, replacing every reference to them with the corresponding symbol.
/// If the given arm pattern is a union, the union is split into multiple arms, one for each symbol in the union.
pub fn remove_captures<F>(ast: Exp<Annot>, rec: F) -> Exp<Annot>
where
    F: Fn(Exp<Annot>) -> Exp<Annot>,
{
    Exp(
        match ast.0 {
            Node::Match { exp, arms } => {
                let mut new_arms = Vec::new();

                for arm in arms {
                    if let Some(id) = arm.catch_id {
                        // Flatten the union into a set of symbols.
                        // It is guaranteed that at this point the pattern is a union of symbols since it has already
                        // been as simplified as possible.
                        let (symbols, pat_loc) = match arm.pat {
                            Pat::Union(union) => {
                                let mut symbols = HashSet::new();
                                assert!(union.union_to_set(&mut symbols));
                                (symbols, union.1 .1)
                            }
                            _ => unreachable!(),
                        };

                        for sym in symbols {
                            let sym = Exp(
                                Node::Symbol(sym.clone()),
                                Annot(Type::Symbol, pat_loc.clone()),
                            );

                            let exp = rec(replace_id(arm.exp.clone(), &id, &sym));

                            new_arms.push(Arm {
                                pat: Pat::Union(sym),
                                catch_id: None,
                                exp,
                            });
                        }
                    } else {
                        new_arms.push(arm);
                    }
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
