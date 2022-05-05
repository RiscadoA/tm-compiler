use crate::annotater::Annot;
use crate::data::{Arm, Exp, Node, Pat, Type};

use std::collections::HashMap;

/// Applies all applications on the AST which don't produce tapes, replacing references to the bound variables with the bound expressions.
pub fn do_applications(ast: Exp<Annot>) -> Exp<Annot> {
    traverse(ast, &HashMap::new())
}

fn traverse(ast: Exp<Annot>, defs: &HashMap<String, Exp<Annot>>) -> Exp<Annot> {
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
                            Pat::Union(exp) => Pat::Union(traverse(exp, defs)),
                            Pat::Any => Pat::Any,
                        };

                        Arm {
                            catch_id: arm.catch_id,
                            pat,
                            exp: traverse(arm.exp, defs),
                        }
                    })
                    .collect(),
            },

            Node::Function { arg, exp } => Node::Function {
                arg,
                exp: Box::new(traverse(*exp, defs)),
            },

            Node::Application { func, arg } => {
                let func = traverse(*func, defs);
                let arg = traverse(*arg, defs);

                let (arg_t, ret_t) = if let Type::Function { arg, ret } = &func.1 .0 {
                    (arg, ret)
                } else {
                    panic!("Expected function type, got {:?}", func.1 .0);
                };

                if &**arg_t != &Type::Tape || &**ret_t != &Type::Tape {
                    if let Node::Function { arg: arg_id, exp } = func.0 {
                        let mut defs = defs.clone();
                        defs.insert(arg_id, arg);
                        return traverse(*exp, &defs);
                    }
                }

                Node::Application {
                    func: Box::new(traverse(func, defs)),
                    arg: Box::new(arg),
                }
            }

            Node::Identifier(id) => match defs.get(&id) {
                Some(exp) => return exp.clone(),
                None => Node::Identifier(id),
            },

            n => n,
        },
        ast.1,
    )
}
