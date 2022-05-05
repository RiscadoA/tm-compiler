use crate::data::{Exp, Node, Pat};

/// Removes trivial applications from the AST:
/// - identity functions: `(x: x) y` -> `y`
/// - application functions: `(x: f x) y` -> `f y`
pub fn remove_trivial<Annot>(ast: Exp<Annot>, changed: &mut bool) -> Exp<Annot>
where
    Annot: Clone,
{
    Exp(
        match ast.0 {
            Node::Union { lhs, rhs } => Node::Union {
                lhs: Box::new(remove_trivial(*lhs, changed)),
                rhs: Box::new(remove_trivial(*rhs, changed)),
            },

            Node::Match { exp, arms } => Node::Match {
                exp: Box::new(remove_trivial(*exp, changed)),
                arms: arms
                    .into_iter()
                    .map(|mut arm| {
                        arm.pat = match arm.pat {
                            Pat::Union(exp) => Pat::Union(remove_trivial(exp, changed)),
                            Pat::Any => Pat::Any,
                        };
                        arm.exp = remove_trivial(arm.exp, changed);
                        arm
                    })
                    .collect(),
            },

            Node::Function { arg, exp } => Node::Function {
                arg,
                exp: Box::new(remove_trivial(*exp, changed)),
            },

            Node::Application { func, arg } => {
                let func = remove_trivial(*func, changed);
                let arg = remove_trivial(*arg, changed);

                if is_identity(&func) {
                    *changed = true;
                    arg.0
                } else if let Some(func) = is_application(&func) {
                    *changed = true;
                    Node::Application {
                        func: Box::new(func),
                        arg: Box::new(arg),
                    }
                } else {
                    Node::Application {
                        func: Box::new(func),
                        arg: Box::new(arg),
                    }
                }
            }

            n => n,
        },
        ast.1,
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
