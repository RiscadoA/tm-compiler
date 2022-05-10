use crate::annotater::Annot;
use crate::data::{Exp, Node, Pat};
use std::collections::HashSet;

/// Merges all arms in the given match expression which have equivalent expressions.
pub fn merge_arms(ast: Exp<Annot>) -> Exp<Annot> {
    Exp(
        match ast.0 {
            Node::Match { exp, mut arms } => {
                let mut new_arms = Vec::new();

                for i in 0..arms.len() {
                    let mut found = false;
                    for j in (i + 1)..arms.len() {
                        if arms[i].exp.eq_ignore_annot(&arms[j].exp) {
                            let (pati, patj) = match (arms[i].pat.clone(), &mut arms[j].pat) {
                                (Pat::Union(pati), Pat::Union(patj)) => (pati, patj),
                                _ => unreachable!(),
                            };

                            let mut set = HashSet::new();
                            assert!(pati.union_to_set(&mut set));
                            assert!(patj.union_to_set(&mut set));
                            *patj = Exp::union_from_set(&set, &patj.1);
                            found = true;
                        }
                    }

                    if !found {
                        new_arms.push(arms[i].clone());
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
