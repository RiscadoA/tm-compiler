use super::Annot;
use crate::data::{Arm, Exp, Node, Pat, TokenLoc, Type};
use std::{collections::HashMap, hash::Hash};

/// Fixes a type annotated AST by resolving every unresolved union type.
pub fn resolve_unions(ast: Exp<Annot>) -> Exp<Annot> {
    // First annotate the AST with IDs for each unresolved union.
    let mut count = 2; // 0 is reserved for Symbol and 1 is reserved for Union.
    let ast = fix_ids(ast, &mut count);

    // Then collect the casts and resolve the unions.
    let mut casts = Vec::new();
    collect_casts(&ast, &mut casts, &HashMap::new(), None);
    let resolved = generate_resolved(count, &casts);

    // Finally, resolve the casts and return the fixed AAST.
    remove_unresolved(ast, &resolved)
}

/// Fixes the ID of a type.
fn fix_ids_in_type(t: Type, count: &mut usize) -> Type {
    match t {
        Type::Function { arg, ret } => Type::Function {
            arg: Box::new(fix_ids_in_type(*arg, count)),
            ret: Box::new(fix_ids_in_type(*ret, count)),
        },
        Type::UnresolvedUnion(0) => {
            *count += 1;
            Type::UnresolvedUnion(*count - 1)
        }
        t => t,
    }
}

/// Fixes the IDs of the unresolved unions in the AST.
fn fix_ids(exp: Exp<Annot>, count: &mut usize) -> Exp<Annot> {
    // Generate ID for the current expression.
    let annot = Annot(fix_ids_in_type(exp.1 .0, count), exp.1 .1);

    Exp(
        match exp.0 {
            Node::Identifier(id) => Node::Identifier(id),
            Node::Symbol(sym) => Node::Symbol(sym),
            Node::Union { lhs, rhs } => Node::Union {
                lhs: Box::new(fix_ids(*lhs, count)),
                rhs: Box::new(fix_ids(*rhs, count)),
            },
            Node::Match { exp, arms } => Node::Match {
                exp: Box::new(fix_ids(*exp, count)),
                arms: arms
                    .into_iter()
                    .map(|arm| Arm {
                        pat: match arm.pat {
                            Pat::Union(exp) => Pat::Union(fix_ids(exp, count)),
                            Pat::Any => Pat::Any,
                        },
                        catch_id: arm.catch_id,
                        exp: fix_ids(arm.exp, count),
                    })
                    .collect(),
            },
            Node::Function { arg, exp } => Node::Function {
                arg,
                exp: Box::new(fix_ids(*exp, count)),
            },
            Node::Application { func, arg } => Node::Application {
                func: Box::new(fix_ids(*func, count)),
                arg: Box::new(fix_ids(*arg, count)),
            },
            _ => unreachable!(),
        },
        annot,
    )
}

/// Collects a casts from one type to another.
fn collect_casts_in_type(from: Option<&Type>, to: Option<&Type>, casts: &mut Vec<(usize, usize)>) {
    if let Some(from) = from {
        if let Some(to) = to {
            match (from, to) {
                (
                    Type::Function { arg, ret },
                    Type::Function {
                        arg: arg_to,
                        ret: ret_to,
                    },
                ) => {
                    collect_casts_in_type(Some(arg_to), Some(arg), casts);
                    collect_casts_in_type(Some(ret), Some(ret_to), casts);
                }
                (Type::UnresolvedUnion(from), Type::Symbol) => {
                    casts.push((*from, 0));
                }
                (Type::UnresolvedUnion(from), Type::Union) => {
                    casts.push((*from, 1));
                }
                (Type::Symbol, Type::UnresolvedUnion(to)) => {
                    casts.push((0, *to));
                }
                (Type::Union, Type::UnresolvedUnion(to)) => {
                    casts.push((1, *to));
                }
                (Type::UnresolvedUnion(from), Type::UnresolvedUnion(to)) if from != to => {
                    casts.push((*from, *to));
                }
                _ => (),
            }
        }
    }
}

/// Collects every union/symbol to unresolved union or vice versa cast.
fn collect_casts(
    exp: &Exp<Annot>,
    casts: &mut Vec<(usize, usize)>,
    ids: &HashMap<String, Type>,
    ret_t: Option<&Type>,
) {
    match &exp.0 {
        Node::Identifier(id) => {
            if let Some(t) = ids.get(id) {
                collect_casts_in_type(Some(t), Some(&exp.1 .0), casts);
                collect_casts_in_type(Some(&exp.1 .0), ret_t, casts);
            }
        }

        Node::Symbol(sym) => {
            collect_casts_in_type(Some(&Type::Symbol), ret_t, casts);
        }

        Node::Union { lhs, rhs } => {
            collect_casts(&lhs, casts, ids, Some(&Type::Union));
            collect_casts(&rhs, casts, ids, Some(&Type::Union));
            collect_casts_in_type(Some(&Type::Union), ret_t, casts);
        }

        Node::Match {
            exp: match_exp,
            arms,
        } => {
            collect_casts_in_type(Some(&exp.1 .0), ret_t, casts);
            collect_casts(&match_exp, casts, ids, Some(&Type::Symbol));
            for arm in arms {
                if let Pat::Union(exp) = &arm.pat {
                    collect_casts(&exp, casts, ids, Some(&Type::Union));
                }

                if let Some(id) = &arm.catch_id {
                    let mut ids = ids.clone();
                    ids.insert(id.clone(), Type::Symbol);
                    collect_casts(&arm.exp, casts, &ids, Some(&exp.1 .0));
                } else {
                    collect_casts(&arm.exp, casts, ids, Some(&exp.1 .0));
                }
            }
        }

        Node::Function { arg, exp: func_exp } => {
            collect_casts_in_type(Some(&exp.1 .0), ret_t, casts);

            let (func_arg_t, func_ret_t) = match &exp.1 .0 {
                Type::Function { arg, ret } => (arg, ret),
                _ => unreachable!(),
            };

            if func_arg_t.is_unresolved_union() {
                let mut ids = ids.clone();
                ids.insert(arg.clone(), *func_arg_t.clone());
                collect_casts(&func_exp, casts, &ids, Some(&func_ret_t));
            } else {
                collect_casts(&func_exp, casts, ids, Some(&func_ret_t));
            }
        }

        Node::Application { func, arg } => {
            let (func_arg_t, func_ret_t) = match &func.1 .0 {
                Type::Function { arg, ret } => (arg, ret),
                _ => unreachable!(),
            };

            collect_casts_in_type(Some(&arg.1 .0), Some(&func_arg_t), casts);
            collect_casts_in_type(Some(&func_ret_t), Some(&exp.1 .0), casts);
            collect_casts_in_type(Some(&exp.1 .0), ret_t, casts);
            collect_casts(&func, casts, ids, None);
            collect_casts(&arg, casts, ids, None);
        }
        _ => (),
    }
}

/// Generates a type vec with the resolved union types from the casts between them.
/// Every type which casts to symbol is resolved to the symbol type.
/// All other types are resolved to the union type.
fn generate_resolved(count: usize, casts: &[(usize, usize)]) -> Vec<Type> {
    let mut types = vec![Type::Symbol, Type::Union];
    types.resize(count, Type::UnresolvedUnion(0));

    // Find every type which casts to a type which is resolved to symbol, and resolve it.
    loop {
        let mut changed = false;

        for (from, to) in casts {
            if types[*from] == Type::UnresolvedUnion(0) && types[*to] == Type::Symbol {
                types[*from] = Type::Symbol;
                changed = true;
            } else if types[*from] == Type::Union && types[*to] == Type::Symbol {
                unreachable!();
            }
        }

        if !changed {
            break;
        }
    }

    // Resolve remaining unresolved types to union.
    for i in 2..count {
        if types[i] == Type::UnresolvedUnion(0) {
            types[i] = Type::Union;
        }
    }

    types
}

/// Replaces the ID annotations in a type the actual type annotations.
fn remove_unresolved_in_type(t: Type, types: &[Type]) -> Type {
    match t {
        Type::Function { arg, ret } => Type::Function {
            arg: Box::new(remove_unresolved_in_type(*arg, types)),
            ret: Box::new(remove_unresolved_in_type(*ret, types)),
        },
        Type::UnresolvedUnion(id) => types[id].clone(),
        t => t,
    }
}

/// Replaces the ID annotations with the actual type annotations.
fn remove_unresolved(exp: Exp<Annot>, types: &[Type]) -> Exp<Annot> {
    let annot = Annot(remove_unresolved_in_type(exp.1 .0, types), exp.1 .1);

    Exp(
        match exp.0 {
            Node::Identifier(id) => Node::Identifier(id),
            Node::Symbol(sym) => Node::Symbol(sym),
            Node::Union { lhs, rhs } => Node::Union {
                lhs: Box::new(remove_unresolved(*lhs, types)),
                rhs: Box::new(remove_unresolved(*rhs, types)),
            },
            Node::Match { exp, arms } => Node::Match {
                exp: Box::new(remove_unresolved(*exp, types)),
                arms: arms
                    .into_iter()
                    .map(|arm| Arm {
                        pat: match arm.pat {
                            Pat::Union(exp) => Pat::Union(remove_unresolved(exp, types)),
                            Pat::Any => Pat::Any,
                        },
                        catch_id: arm.catch_id,
                        exp: remove_unresolved(arm.exp, types),
                    })
                    .collect(),
            },
            Node::Function { arg, exp } => Node::Function {
                arg,
                exp: Box::new(remove_unresolved(*exp, types)),
            },
            Node::Application { func, arg } => Node::Application {
                func: Box::new(remove_unresolved(*func, types)),
                arg: Box::new(remove_unresolved(*arg, types)),
            },
            _ => unreachable!(),
        },
        annot,
    )
}
