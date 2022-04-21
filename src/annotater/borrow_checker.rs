use super::type_checker::Annot;
use crate::data::{Arm, Exp, Node, Pat, TokenLoc, Type, TypeGraph};

/// Checks if the tape borrow rules are being obeyed.
/// Searches for function applications which take &tape and instead receive tape.
pub fn borrow_check(exp: &Exp<Annot>) -> Result<(), String> {
    match &exp.0 {
        Node::Union { lhs, rhs } => {
            borrow_check(&lhs)?;
            borrow_check(&rhs)
        }

        Node::Match { exp, arms } => {
            borrow_check(&exp)?;
            for arm in arms.iter() {
                match &arm.pat {
                    Pat::Union(exp) => borrow_check(exp)?,
                    _ => (),
                };
                borrow_check(&arm.exp)?;
            }
            Ok(())
        }

        Node::Let { exp, binds } => {
            for bind in binds.iter() {
                borrow_check(&bind.1)?;
            }
            borrow_check(&exp)
        }

        Node::Function { arg, exp } => borrow_check(&exp),

        Node::Application { func, arg } => {
            // If the function receives a &tape, the argument must also be a &tape.
            if let Type::Function { arg: farg, .. } = &func.1 .0 {
                match &**farg {
                    Type::Tape { owned: false } => {
                        if arg.1 .0 != (Type::Tape { owned: false }) {
                            return Err(format!("Application of &tape function at {} received owned tape which violates ownership rules", exp.1.1));
                        }
                    }
                    _ => (),
                }
            }

            borrow_check(func)?;
            borrow_check(arg)
        }

        _ => Ok(()),
    }
}
