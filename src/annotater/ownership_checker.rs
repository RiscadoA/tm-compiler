use super::Annot;
use crate::data::{Arm, Exp, Node, Pat, TokenLoc, Type};
use std::collections::HashSet;

/// Converts tape types to &tape where possible and then checks if the tape ownership rules are met.
pub fn ownership_check(ast: &Exp<Annot>) -> Result<(), String> {
    traverse(ast, &mut HashSet::new(), false)
}

fn traverse(exp: &Exp<Annot>, consumed: &mut HashSet<String>, is_ref: bool) -> Result<(), String> {
    match &exp.0 {
        Node::Identifier(id) if exp.1 .0 == Type::Tape => {
            if consumed.contains(id) {
                return Err(format!(
                    "Tape being used at {} was already consumed",
                    exp.1 .1
                ));
            } else if !is_ref {
                consumed.insert(id.clone());
            }
        }

        Node::Match {
            exp: match_exp,
            arms,
        } => {
            traverse(match_exp, consumed, false)?;
            let initial_set = consumed.clone();
            for arm in arms.iter() {
                let mut arm_set = initial_set.clone();
                if let Some(id) = &arm.catch_id {
                    arm_set.remove(id);
                }
                traverse(&arm.exp, &mut arm_set, is_ref)?;
                consumed.extend(arm_set);
            }
        }

        Node::Function { arg, exp: func_exp } => {
            consumed.remove(arg);
            traverse(func_exp, consumed, false)?;
        }

        Node::Application { func, arg } => {
            let (func_arg_t, func_ret_t) = match &func.1 .0 {
                Type::Function { arg, ret } => (arg, ret),
                _ => unreachable!(),
            };

            match (&**func_arg_t, &**func_ret_t) {
                (Type::Tape, Type::Symbol) => {
                    traverse(func, consumed, false)?;
                    traverse(arg, consumed, true)?;
                }
                (Type::Tape, o) if o != &Type::Tape => {
                    return Err(format!(
                        "Function at {} receives tape as argument but returns {}, while only tape & symbol are allowed",
                        func.1.1,
                        o
                    ));
                }
                _ => {
                    traverse(func, consumed, false)?;
                    traverse(arg, consumed, false)?;
                }
            }
        }

        _ => {}
    }

    Ok(())
}
