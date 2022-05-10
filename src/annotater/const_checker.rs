use super::Annot;
use crate::data::{Exp, Node, Pat};
use std::collections::HashSet;

/// Checks if all match patterns are constant.
pub fn const_check(ast: &Exp<Annot>) -> Result<(), String> {
    is_const(ast, &HashSet::new())?;
    Ok(())
}

fn is_const(exp: &Exp<Annot>, const_exps: &HashSet<String>) -> Result<bool, String> {
    match &exp.0 {
        Node::Identifier(id) => Ok(const_exps.contains(id)),
        Node::Symbol(_) => Ok(true),
        Node::Accept | Node::Reject | Node::Abort => Ok(true),
        Node::Union { lhs, rhs } => {
            let mut ret = true;
            ret &= is_const(lhs, const_exps)?;
            ret &= is_const(rhs, const_exps)?;
            Ok(ret)
        }
        Node::Match { exp, arms } => {
            let mut ret = is_const(exp, const_exps)?;
            for arm in arms {
                if let Pat::Union(exp) = &arm.pat {
                    if !is_const(exp, const_exps)? {
                        return Err(format!("Match pattern at {} is not constant", exp.1 .1));
                    }
                }

                if let Some(id) = &arm.catch_id {
                    let mut const_exps = const_exps.clone();
                    const_exps.insert(id.clone());
                    ret = ret && is_const(&arm.exp, &const_exps)?;
                } else {
                    ret = ret && is_const(&arm.exp, const_exps)?;
                }
            }
            Ok(ret)
        }
        Node::Function { arg, exp } => {
            let mut const_exps = const_exps.clone();
            const_exps.insert(arg.clone());
            is_const(exp, &const_exps)
        }
        Node::Application { func, arg } => {
            let mut ret = true;
            ret &= is_const(func, const_exps)?;
            ret &= is_const(arg, const_exps)?;
            Ok(ret)
        }
        _ => unreachable!(),
    }
}
