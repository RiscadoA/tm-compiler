use crate::data::{Exp, Node};

/// If the given expression is a let expression, it is simplified into function applications.
pub fn remove_lets<Annot>(ast: Exp<Annot>) -> Exp<Annot>
where
    Annot: Clone,
{
    Exp(
        match ast.0 {
            Node::Let { mut exp, binds } => {
                for (id, bind) in binds.into_iter().rev() {
                    let annot = bind.1.clone();
                    exp = Box::new(Exp(
                        Node::Application {
                            func: Box::new(Exp(Node::Function { arg: id, exp }, annot.clone())),
                            arg: Box::new(bind),
                        },
                        annot,
                    ));
                }
                exp.0
            }

            n => n,
        },
        ast.1,
    )
}
