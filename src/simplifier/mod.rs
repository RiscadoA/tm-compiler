use crate::data::Exp;
use crate::annotater::Annot;

mod applier;
mod id_dedup;
mod let_remover;

pub fn simplify(ast: Exp<Annot>) -> Exp<Annot>
where
    Annot: Clone,
{
    let ast = let_remover::remove_lets(ast);
    let ast = id_dedup::dedup_ids(ast);
    let ast = applier::do_applications(ast);
    ast
}
