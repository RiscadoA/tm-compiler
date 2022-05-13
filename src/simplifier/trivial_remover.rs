use crate::data::{Exp, Node, Pat};

/// If the given expression is a trivial application of a function, simplifies it.
/// - identity functions: `(x: x) y` -> `y`
/// - application functions: `(x: f x) y` -> `f y`
/// - unused functions: `(x: a) y` -> `a`
pub fn remove_trivial<Annot>(exp: Exp<Annot>) -> Exp<Annot>
where
    Annot: Clone,
{
    Exp(
        match exp.0 {
            Node::Application { func, arg } => {
                if is_identity(&func) {
                    arg.0
                } else if let Some(func) = is_application(&func) {
                    Node::Application {
                        func: Box::new(func),
                        arg: Box::new(*arg),
                    }
                } else if let Some(exp) = is_unused(&func) {
                    exp.0
                } else {
                    Node::Application {
                        func: Box::new(*func),
                        arg: Box::new(*arg),
                    }
                }
            }

            n => n,
        },
        exp.1,
    )
}

/// Checks if a function expression is an identity function.
fn is_identity<Annot>(exp: &Exp<Annot>) -> bool {
    match &exp.0 {
        Node::Function { arg, exp } => match &exp.0 {
            Node::Identifier(id) if id == arg => true,
            _ => false,
        },
        _ => false,
    }
}

/// Checks if a function expression is an application function, and if so returns the function being applied.
fn is_application<Annot>(exp: &Exp<Annot>) -> Option<Exp<Annot>>
where
    Annot: Clone,
{
    match &exp.0 {
        Node::Function { arg: arg_id, exp } => match &exp.0 {
            Node::Application { func, arg } => match &arg.0 {
                Node::Identifier(id) if id == arg_id => Some(*func.clone()),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

/// Checks if a function expression is an unused function, and if so returns the function expression.
fn is_unused<Annot>(exp: &Exp<Annot>) -> Option<Exp<Annot>>
where
    Annot: Clone,
{
    match &exp.0 {
        Node::Function { arg, exp } if !uses_id(exp, arg) => Some(*exp.clone()),
        _ => None,
    }
}

/// Checks if the given identifier is used in the given expression.
fn uses_id<Annot>(ast: &Exp<Annot>, id: &str) -> bool {
    match &ast.0 {
        Node::Identifier(id2) if id == id2 => true,
        Node::Union { lhs, rhs } => uses_id(lhs, id) || uses_id(rhs, id),
        Node::Match {
            exp: match_exp,
            arms,
        } => {
            uses_id(match_exp, id)
                || arms.iter().any(|arm| {
                    (arm.catch_id != Some(id.to_owned()) && uses_id(&arm.exp, id))
                        || match &arm.pat {
                            Pat::Union(exp) => uses_id(exp, id),
                            Pat::Any => unreachable!(),
                        }
                })
        }
        Node::Function { arg, exp } if arg != id => uses_id(exp, id),
        Node::Application { func, arg } => uses_id(func, id) || uses_id(arg, id),
        _ => false,
    }
}
