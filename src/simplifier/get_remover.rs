use crate::annotater::Annot;
use crate::data::{Arm, Exp, Node, Pat, Type};
use std::collections::HashSet;

/// If the current expression is a get application, it is replaced by a match expression with the tape as its
/// expression.
pub fn remove_gets(ast: Exp<Annot>, alphabet: &HashSet<String>) -> Exp<Annot> {
    Exp(
        match ast.0 {
            Node::Application { func, arg } => match func.0 {
                Node::Identifier(id) if id == "get" => Node::Match {
                    exp: arg,
                    arms: alphabet
                        .iter()
                        .map(|sym| {
                            let sym = Exp(
                                Node::Symbol(sym.clone()),
                                Annot(Type::Symbol, ast.1 .1.clone()),
                            );

                            Arm {
                                pat: Pat::Union(sym.clone()),
                                catch_id: None,
                                exp: sym,
                            }
                        })
                        .collect(),
                },
                _ => Node::Application { func, arg },
            },

            n => n,
        },
        ast.1,
    )
}
