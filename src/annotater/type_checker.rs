use crate::data::{Arm, Exp, Node, Pat, TokenLoc, Type, TypeTable};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Annot(pub Type, pub TokenLoc);

/// Annotates an AST with types, checking for type errors.
/// All remaining unresolved types
pub fn type_check(ast: Exp<TokenLoc>) -> Result<Exp<Annot>, String> {
    let mut type_table = TypeTable::new();
    let ast = check_exp(
        ast,
        &define_builtin_functions(),
        &mut type_table,
        &Type::Function {
            arg: Box::new(Type::Tape { owned: true }),
            ret: Box::new(Type::Tape { owned: true }),
        },
    )?;
    resolve_exp(ast, &mut type_table, false)
}

/// Checks the types of an expression.
fn check_exp(
    exp: Exp<TokenLoc>,
    vars: &HashMap<String, Type>,
    type_table: &mut TypeTable,
    ret_t: &Type,
) -> Result<Exp<Annot>, String> {
    match exp.0 {
        Node::Identifier(id) => {
            let t = vars
                .get(&id)
                .ok_or_else(|| format!("Undefined identifier {} at {}", id, exp.1))?;
            type_table.cast(t, ret_t, &exp.1)?;
            Ok(Exp(Node::Identifier(id), Annot(t.clone(), exp.1)))
        }

        Node::Symbol(sym) => {
            type_table.cast(&Type::Symbol, ret_t, &exp.1)?;
            Ok(Exp(Node::Symbol(sym), Annot(Type::Symbol, exp.1)))
        }

        Node::Union { lhs, rhs } => {
            let lhs = check_exp(*lhs, vars, type_table, &Type::Union)?;
            let rhs = check_exp(*rhs, vars, type_table, &Type::Union)?;
            type_table.cast(&Type::Union, ret_t, &exp.1)?;
            Ok(Exp(
                Node::Union {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                Annot(Type::Union, exp.1),
            ))
        }

        Node::Match {
            exp: match_exp,
            arms,
        } => {
            let match_exp = check_exp(*match_exp, vars, type_table, &Type::Symbol)?;

            let mut new_arms = Vec::new();
            for arm in arms.into_iter() {
                let pat = match arm.pat {
                    Pat::Union(exp) => {
                        let exp = check_exp(exp, vars, type_table, &Type::Union)?;
                        Pat::Union(exp)
                    }
                    Pat::Any => Pat::Any,
                };

                let mut vars = vars.clone();
                if let Some(catch_id) = &arm.catch_id {
                    vars.insert(catch_id.clone(), Type::Symbol);
                }

                new_arms.push(Arm {
                    catch_id: arm.catch_id,
                    pat,
                    exp: check_exp(arm.exp, &vars, type_table, ret_t)?,
                });
            }

            Ok(Exp(
                Node::Match {
                    exp: Box::new(match_exp),
                    arms: new_arms,
                },
                Annot(ret_t.clone(), exp.1),
            ))
        }

        Node::Let {
            exp: let_exp,
            binds,
        } => {
            let mut vars = vars.clone();

            let mut new_binds = Vec::new();
            for (id, exp) in binds {
                let t = type_table.push();
                let exp = check_exp(exp, &vars, type_table, &t)?;
                vars.insert(id.clone(), t);
                new_binds.push((id, exp));
            }

            Ok(Exp(
                Node::Let {
                    exp: Box::new(check_exp(*let_exp, &vars, type_table, ret_t)?),
                    binds: new_binds,
                },
                Annot(ret_t.clone(), exp.1),
            ))
        }

        Node::Function {
            arg,
            exp: function_exp,
        } => {
            let func_arg_t = type_table.push();
            let func_ret_t = type_table.push();

            let mut vars = vars.clone();
            vars.insert(arg.clone(), func_arg_t.clone());
            let function_exp = check_exp(*function_exp, &vars, type_table, &func_ret_t)?;

            let func_t = Type::Function {
                arg: Box::new(func_arg_t),
                ret: Box::new(func_ret_t),
            };
            type_table.cast(&func_t, ret_t, &exp.1)?;

            Ok(Exp(
                Node::Function {
                    arg,
                    exp: Box::new(function_exp),
                },
                Annot(func_t, exp.1),
            ))
        }

        Node::Application { func, arg } => {
            let arg_t = type_table.push();
            let arg = check_exp(*arg, vars, type_table, &arg_t)?;

            let func_t = Type::Function {
                arg: Box::new(arg_t),
                ret: Box::new(ret_t.clone()),
            };
            let func = check_exp(*func, vars, type_table, &func_t)?;

            Ok(Exp(
                Node::Application {
                    func: Box::new(func),
                    arg: Box::new(arg),
                },
                Annot(ret_t.clone(), exp.1),
            ))
        }
    }
}

/// Resolves all unresolved types in an expression.
fn resolve_exp(
    exp: Exp<Annot>,
    type_table: &mut TypeTable,
    allow_unresolved: bool,
) -> Result<Exp<Annot>, String> {
    let exp = match exp.0 {
        Node::Union { lhs, rhs } => Exp(
            Node::Union {
                lhs: Box::new(resolve_exp(*lhs, type_table, false)?),
                rhs: Box::new(resolve_exp(*rhs, type_table, false)?),
            },
            Annot(Type::Union, exp.1 .1),
        ),

        Node::Match {
            exp: match_exp,
            arms,
        } => {
            let match_exp = resolve_exp(*match_exp, type_table, false)?;

            let mut new_arms = Vec::new();
            for arm in arms.into_iter() {
                let pat = match arm.pat {
                    Pat::Union(exp) => {
                        let exp = resolve_exp(exp, type_table, false)?;
                        Pat::Union(exp)
                    }
                    Pat::Any => Pat::Any,
                };

                let exp = resolve_exp(arm.exp, type_table, allow_unresolved)?;
                new_arms.push(Arm {
                    catch_id: arm.catch_id,
                    pat,
                    exp,
                });
            }

            Exp(
                Node::Match {
                    exp: Box::new(match_exp.clone()),
                    arms: new_arms,
                },
                Annot(type_table.resolve(&exp.1 .0), exp.1 .1),
            )
        }

        Node::Let {
            exp: body_exp,
            binds,
        } => {
            let mut new_binds = Vec::new();
            for (id, exp) in binds {
                let exp = resolve_exp(exp, type_table, true)?;
                new_binds.push((id, exp));
            }

            Exp(
                Node::Let {
                    exp: Box::new(resolve_exp(*body_exp, type_table, allow_unresolved)?),
                    binds: new_binds,
                },
                Annot(type_table.resolve(&exp.1 .0), exp.1 .1),
            )
        }

        Node::Function { arg, exp: body_exp } => Exp(
            Node::Function {
                arg,
                exp: Box::new(resolve_exp(*body_exp, type_table, allow_unresolved)?),
            },
            Annot(type_table.resolve(&exp.1 .0), exp.1 .1),
        ),

        Node::Application { func, arg } => Exp(
            Node::Application {
                func: Box::new(resolve_exp(*func, type_table, allow_unresolved)?),
                arg: Box::new(resolve_exp(*arg, type_table, allow_unresolved)?),
            },
            Annot(type_table.resolve(&exp.1 .0), exp.1 .1),
        ),

        node => Exp(node, Annot(type_table.resolve(&exp.1 .0), exp.1 .1)),
    };

    if !exp.1 .0.is_resolved() && !allow_unresolved {
        Err(format!(
            "Couldn't resolve type {} at {}",
            exp.1 .0, exp.1 .1
        ))
    } else {
        Ok(exp)
    }
}

/// Defines the types of the built-in functions.
fn define_builtin_functions() -> HashMap<String, Type> {
    let mut vars = HashMap::new();

    // set Symbol Tape -> Tape
    vars.insert(
        "set".to_owned(),
        Type::Function {
            arg: Box::new(Type::Symbol),
            ret: Box::new(Type::Function {
                arg: Box::new(Type::Tape { owned: true }),
                ret: Box::new(Type::Tape { owned: true }),
            }),
        },
    );

    // get &Tape -> Symbol
    vars.insert(
        "get".to_owned(),
        Type::Function {
            arg: Box::new(Type::Tape { owned: false }),
            ret: Box::new(Type::Symbol),
        },
    );

    // next Tape -> Tape
    vars.insert(
        "next".to_owned(),
        Type::Function {
            arg: Box::new(Type::Tape { owned: true }),
            ret: Box::new(Type::Tape { owned: true }),
        },
    );

    // prev Tape -> Tape
    vars.insert(
        "prev".to_owned(),
        Type::Function {
            arg: Box::new(Type::Tape { owned: true }),
            ret: Box::new(Type::Tape { owned: true }),
        },
    );

    // Y ((Tape -> Tape) -> Tape -> Tape) -> (Tape -> Tape)
    vars.insert(
        "Y".to_owned(),
        Type::Function {
            arg: Box::new(Type::Function {
                arg: Box::new(Type::Function {
                    arg: Box::new(Type::Tape { owned: true }),
                    ret: Box::new(Type::Tape { owned: true }),
                }),
                ret: Box::new(Type::Function {
                    arg: Box::new(Type::Tape { owned: true }),
                    ret: Box::new(Type::Tape { owned: true }),
                }),
            }),
            ret: Box::new(Type::Function {
                arg: Box::new(Type::Tape { owned: true }),
                ret: Box::new(Type::Tape { owned: true }),
            }),
        },
    );

    vars
}

impl fmt::Display for Annot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_check_simple() {
        todo!();
    }
}
