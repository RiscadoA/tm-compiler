use crate::data::{Arm, Exp, Node, Pat, TokenLoc, Type, TypeGraph};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Annot(pub Type, pub TokenLoc);

/// Annotates an AST with types, checking for type errors.
/// All remaining unresolved types
pub fn type_check(ast: Exp<TokenLoc>) -> Result<Exp<Annot>, String> {
    let mut type_graph = TypeGraph::new();
    let ast = check_exp(ast, &define_builtin_functions(), &mut type_graph)?;
    type_graph.cast(
        &ast.1 .0,
        &Type::Function {
            arg: Box::new(Type::Tape { owned: true }),
            ret: Box::new(Type::Tape { owned: true }),
        },
        &ast.1 .1,
    );
    type_graph.resolve()?;
    resolve_exp(ast, &mut type_graph, false)
}

/// Checks the types of an expression.
fn check_exp(
    exp: Exp<TokenLoc>,
    vars: &HashMap<String, (bool, Type)>,
    type_graph: &mut TypeGraph,
) -> Result<Exp<Annot>, String> {
    match exp.0 {
        Node::Identifier(id) => {
            let src_typ = vars
                .get(&id)
                .ok_or_else(|| format!("Undefined identifier {} at {}", id, exp.1))?;
            if src_typ.0 {
                Ok(Exp(Node::Identifier(id), Annot(src_typ.1.clone(), exp.1)))
            } else {
                let typ = type_graph.push();
                type_graph.cast(&src_typ.1, &typ, &exp.1);
                Ok(Exp(Node::Identifier(id), Annot(typ, exp.1)))
            }
        }

        Node::Symbol(sym) => Ok(Exp(Node::Symbol(sym), Annot(Type::Symbol, exp.1))),

        Node::Union { lhs, rhs } => {
            let lhs = check_exp(*lhs, vars, type_graph)?;
            let rhs = check_exp(*rhs, vars, type_graph)?;

            type_graph.cast(&lhs.1 .0, &Type::Union, &lhs.1 .1);
            type_graph.cast(&rhs.1 .0, &Type::Union, &rhs.1 .1);

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
            let match_exp = check_exp(*match_exp, vars, type_graph)?;
            type_graph.cast(&match_exp.1 .0, &Type::Symbol, &match_exp.1 .1);
            let match_ret = type_graph.push();
            let mut new_arms = Vec::new();
            for arm in arms.into_iter() {
                let pat = match arm.pat {
                    Pat::Union(exp) => {
                        let exp = check_exp(exp, vars, type_graph)?;
                        type_graph.cast(&exp.1 .0, &Type::Union, &exp.1 .1);
                        Pat::Union(exp)
                    }
                    Pat::Any => Pat::Any,
                };

                let mut vars = vars.clone();
                if let Some(catch_id) = &arm.catch_id {
                    vars.insert(catch_id.clone(), (true, Type::Symbol));
                }

                let exp = check_exp(arm.exp, &vars, type_graph)?;
                type_graph.cast(&exp.1 .0, &match_ret, &exp.1 .1);

                new_arms.push(Arm {
                    catch_id: arm.catch_id,
                    pat,
                    exp,
                });
            }

            Ok(Exp(
                Node::Match {
                    exp: Box::new(match_exp),
                    arms: new_arms,
                },
                Annot(match_ret, exp.1),
            ))
        }

        Node::Let {
            exp: body_exp,
            binds,
        } => {
            let mut vars = vars.clone();

            let mut new_binds = Vec::new();
            for (id, exp) in binds {
                let exp = check_exp(exp, &vars, type_graph)?;
                vars.insert(id.clone(), (true, exp.1 .0.clone()));
                new_binds.push((id, exp));
            }

            let body_exp = check_exp(*body_exp, &vars, type_graph)?;
            let body_exp_t = body_exp.1 .0.clone();

            Ok(Exp(
                Node::Let {
                    exp: Box::new(body_exp),
                    binds: new_binds,
                },
                Annot(body_exp_t, exp.1),
            ))
        }

        Node::Function { arg, exp: body_exp } => {
            let mut vars = vars.clone();
            let arg_t = type_graph.push();
            vars.insert(arg.clone(), (false, arg_t.clone()));

            let body_exp = check_exp(*body_exp, &vars, type_graph)?;
            let body_exp_t = body_exp.1 .0.clone();

            Ok(Exp(
                Node::Function {
                    arg,
                    exp: Box::new(body_exp),
                },
                Annot(
                    Type::Function {
                        arg: Box::new(arg_t),
                        ret: Box::new(body_exp_t),
                    },
                    exp.1,
                ),
            ))
        }

        Node::Application { func, arg } => {
            let func = check_exp(*func, vars, type_graph)?;
            let arg = check_exp(*arg, vars, type_graph)?;

            let arg_t = type_graph.push();
            let ret_t = type_graph.push();

            type_graph.cast(
                &func.1 .0,
                &Type::Function {
                    arg: Box::new(arg_t.clone()),
                    ret: Box::new(ret_t.clone()),
                },
                &func.1 .1,
            );

            type_graph.cast(&arg.1 .0, &arg_t, &arg.1 .1);

            Ok(Exp(
                Node::Application {
                    func: Box::new(func),
                    arg: Box::new(arg),
                },
                Annot(ret_t, exp.1),
            ))
        }
    }
}

/// Resolves all unresolved types in an expression.
fn resolve_exp(
    exp: Exp<Annot>,
    type_graph: &TypeGraph,
    allow_unresolved: bool,
) -> Result<Exp<Annot>, String> {
    match exp.0 {
        Node::Union { lhs, rhs } => {
            let lhs = resolve_exp(*lhs, type_graph, false)?;
            let rhs = resolve_exp(*rhs, type_graph, false)?;

            Ok(Exp(
                Node::Union {
                    lhs: Box::new(lhs),
                    rhs: Box::new(rhs),
                },
                Annot(Type::Union, exp.1 .1),
            ))
        }

        Node::Match {
            exp: match_exp,
            arms,
        } => {
            let match_exp = resolve_exp(*match_exp, type_graph, false)?;
            let mut new_arms = Vec::new();
            for arm in arms.into_iter() {
                let pat = match arm.pat {
                    Pat::Union(exp) => {
                        let exp = resolve_exp(exp, type_graph, false)?;
                        Pat::Union(exp)
                    }
                    Pat::Any => Pat::Any,
                };

                let exp = resolve_exp(arm.exp, type_graph, allow_unresolved)?;
                new_arms.push(Arm {
                    catch_id: arm.catch_id,
                    pat,
                    exp,
                });
            }

            Ok(Exp(
                Node::Match {
                    exp: Box::new(match_exp.clone()),
                    arms: new_arms,
                },
                Annot(type_graph.get(&exp.1 .0), exp.1 .1),
            ))
        }

        Node::Let {
            exp: body_exp,
            binds,
        } => {
            let mut new_binds = Vec::new();
            for (id, exp) in binds {
                let exp = resolve_exp(exp, type_graph, true)?;
                new_binds.push((id, exp));
            }

            let body_exp = resolve_exp(*body_exp, type_graph, allow_unresolved)?;
            Ok(Exp(
                Node::Let {
                    exp: Box::new(body_exp.clone()),
                    binds: new_binds,
                },
                Annot(body_exp.1 .0, exp.1 .1),
            ))
        }

        Node::Function { arg, exp: body_exp } => {
            let body_exp = resolve_exp(*body_exp, type_graph, allow_unresolved)?;
            Ok(Exp(
                Node::Function {
                    arg,
                    exp: Box::new(body_exp.clone()),
                },
                Annot(type_graph.get(&exp.1 .0), exp.1 .1),
            ))
        }

        Node::Application { func, arg } => {
            let func = resolve_exp(*func, type_graph, allow_unresolved)?;
            let arg = resolve_exp(*arg, type_graph, allow_unresolved)?;

            Ok(Exp(
                Node::Application {
                    func: Box::new(func),
                    arg: Box::new(arg.clone()),
                },
                Annot(type_graph.get(&exp.1 .0), exp.1 .1),
            ))
        }

        node => {
            let t = type_graph.get(&exp.1 .0);
            if !t.is_resolved() && !allow_unresolved {
                return Err(format!("Couldn't resolve type {} at {}", t, exp.1 .1));
            }
            Ok(Exp(node, Annot(t, exp.1 .1)))
        }
    }
}

/// Defines the types of the built-in functions.
fn define_builtin_functions() -> HashMap<String, (bool, Type)> {
    let mut vars = HashMap::new();

    // set Symbol Tape -> Tape
    vars.insert(
        "set".to_owned(),
        (
            true,
            Type::Function {
                arg: Box::new(Type::Symbol),
                ret: Box::new(Type::Function {
                    arg: Box::new(Type::Tape { owned: true }),
                    ret: Box::new(Type::Tape { owned: true }),
                }),
            },
        ),
    );

    // get &Tape -> Symbol
    vars.insert(
        "get".to_owned(),
        (
            true,
            Type::Function {
                arg: Box::new(Type::Tape { owned: false }),
                ret: Box::new(Type::Symbol),
            },
        ),
    );

    // next Tape -> Tape
    vars.insert(
        "next".to_owned(),
        (
            true,
            Type::Function {
                arg: Box::new(Type::Tape { owned: true }),
                ret: Box::new(Type::Tape { owned: true }),
            },
        ),
    );

    // prev Tape -> Tape
    vars.insert(
        "prev".to_owned(),
        (
            true,
            Type::Function {
                arg: Box::new(Type::Tape { owned: true }),
                ret: Box::new(Type::Tape { owned: true }),
            },
        ),
    );

    // Y ((Tape -> Tape) -> Tape -> Tape) -> (Tape -> Tape)
    vars.insert(
        "Y".to_owned(),
        (
            true,
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
        ),
    );

    vars
}

impl fmt::Display for Annot {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
