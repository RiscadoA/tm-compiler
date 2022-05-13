use crate::data::{Arm, Exp, Node, Pat};

use std::collections::HashMap;

/// Changes all duplicate identifiers to unique identifiers. This is done by prepending duplicate identifiers with a _.
pub fn dedup_ids<Annot>(ast: Exp<Annot>) -> Exp<Annot>
where
    Annot: Clone,
{
    traverse(ast, &HashMap::new())
}

fn traverse<Annot>(ast: Exp<Annot>, renames: &HashMap<String, String>) -> Exp<Annot>
where
    Annot: Clone,
{
    Exp(
        match ast.0 {
            Node::Identifier(id) => Node::Identifier(get_id(renames, id)),

            Node::Union { lhs, rhs } => Node::Union {
                lhs: Box::new(traverse(*lhs, renames)),
                rhs: Box::new(traverse(*rhs, renames)),
            },

            Node::Match { exp, arms } => Node::Match {
                exp: Box::new(traverse(*exp, renames)),
                arms: arms
                    .into_iter()
                    .map(|arm| {
                        let pat = match arm.pat {
                            Pat::Union(exp) => Pat::Union(traverse(exp, renames)),
                            Pat::Any => Pat::Any,
                        };

                        let mut renames = renames.clone();
                        let catch_id = if let Some(id) = arm.catch_id {
                            Some(push_id(&mut renames, id))
                        } else {
                            None
                        };

                        Arm {
                            catch_id,
                            pat,
                            exp: traverse(arm.exp, &renames),
                        }
                    })
                    .collect(),
            },

            Node::Function { arg, exp } => {
                let mut renames = renames.clone();
                let arg = push_id(&mut renames, arg);
                Node::Function {
                    arg,
                    exp: Box::new(traverse(*exp, &renames)),
                }
            }

            Node::Application { func, arg } => Node::Application {
                func: Box::new(traverse(*func, renames)),
                arg: Box::new(traverse(*arg, renames)),
            },

            n => n,
        },
        ast.1,
    )
}

/// Pushes a new identifier to the renaming map.
fn push_id(renames: &mut HashMap<String, String>, id: String) -> String {
    match renames.get(&id) {
        Some(renamed) => {
            let renamed = format!("_{}", renamed);
            renames.insert(id.clone(), renamed.clone());
            renamed
        }
        None => {
            renames.insert(id.clone(), id.clone());
            id
        }
    }
}

/// Gets the renamed identifier from the renaming map.
fn get_id(renames: &HashMap<String, String>, id: String) -> String {
    match renames.get(&id) {
        Some(renamed) => renamed.clone(),
        None => id,
    }
}
