use crate::data::{Arm, Exp, Node, Pat};

use std::collections::HashMap;

/// Removes all let bindings from the AST, replacing references to the bound variables with the bound expressions.
pub fn remove_lets<Annot>(ast: Exp<Annot>) -> Exp<Annot>
where
    Annot: Clone,
{
    traverse(ast, &HashMap::new())
}

fn traverse<Annot>(ast: Exp<Annot>, defs: &HashMap<String, Exp<Annot>>) -> Exp<Annot>
where
    Annot: Clone,
{
    Exp(
        match ast.0 {
            Node::Union { lhs, rhs } => Node::Union {
                lhs: Box::new(traverse(*lhs, defs)),
                rhs: Box::new(traverse(*rhs, defs)),
            },

            Node::Match { exp, arms } => Node::Match {
                exp: Box::new(traverse(*exp, defs)),
                arms: arms
                    .into_iter()
                    .map(|arm| {
                        let pat = match arm.pat {
                            Pat::Union(exp) => Pat::Union(traverse(exp, &defs)),
                            Pat::Any => Pat::Any,
                        };

                        let mut defs = defs.clone();
                        if let Some(id) = &arm.catch_id {
                            defs.remove(id);
                        }

                        Arm {
                            catch_id: arm.catch_id,
                            pat,
                            exp: traverse(arm.exp, &defs),
                        }
                    })
                    .collect(),
            },

            Node::Let { exp, binds } => {
                let mut defs = defs.clone();
                for (id, exp) in binds {
                    defs.insert(id, traverse(exp, &defs));
                }
                return traverse(*exp, &defs);
            }

            Node::Function { arg, exp } => {
                let mut defs = defs.clone();
                defs.remove(&arg);
                Node::Function {
                    arg,
                    exp: Box::new(traverse(*exp, &defs)),
                }
            },

            Node::Application { func, arg } => Node::Application {
                func: Box::new(traverse(*func, defs)),
                arg: Box::new(traverse(*arg, defs)),
            },

            Node::Identifier(id) => match defs.get(&id) {
                Some(exp) => return exp.clone(),
                None => Node::Identifier(id),
            },

            n => n,
        },
        ast.1,
    )
}
