use crate::data::{Exp, Node, Pat};

/// If the given expression is a match expression which matches an abort expression, it is turned into an abort.
/// Otherwise, if the match expression contains arms which match abort expressions, these arms are removed.
pub fn spread_aborts<Annot>(ast: Exp<Annot>) -> Exp<Annot>
where
    Annot: Clone,
{
    Exp(
        match ast.0 {
            Node::Match { exp, .. } if matches!(*exp, Exp(Node::Abort, _)) => Node::Abort,
            Node::Match { exp, mut arms } => {
                arms.retain(|arm| !matches!(arm.pat, Pat::Union(Exp(Node::Abort, _))));
                if arms.is_empty() {
                    Node::Abort
                } else {
                    Node::Match { exp, arms }
                }
            }
            n => n,
        },
        ast.1,
    )
}
