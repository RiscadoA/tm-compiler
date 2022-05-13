use super::id_replacer::replace_id;
use crate::annotater::Annot;
use crate::data::{Exp, Node, Type};

/// Applies the given expression if it is either:
/// - a non tape -> tape function application
/// - a tape -> tape function application with an identifier as argument
pub fn apply<F>(ast: Exp<Annot>, rec: F) -> Exp<Annot>
where
    F: Fn(Exp<Annot>) -> Exp<Annot>,
{
    Exp(
        match ast.0 {
            Node::Application { func, arg } => {
                let (arg_t, ret_t) = if let Type::Function { arg, ret } = &func.1 .0 {
                    (arg, ret)
                } else {
                    unreachable!()
                };

                if &**arg_t != &Type::Tape
                    || &**ret_t != &Type::Tape
                    || matches!(arg.0, Node::Identifier(_))
                {
                    if let Node::Function { arg: arg_id, exp } = func.0 {
                        return rec(replace_id(*exp, &arg_id, &arg));
                    }
                }

                Node::Application { func, arg }
            }

            n => n,
        },
        ast.1,
    )
}
