use crate::annotater::Annot;
use crate::data::Exp;

mod applier;
mod id_dedup;
mod let_remover;
mod match_mover;

pub fn simplify(ast: Exp<Annot>) -> Exp<Annot>
where
    Annot: Clone,
{
    let ast = let_remover::remove_lets(ast);
    let ast = id_dedup::dedup_ids(ast);
    let ast = applier::do_applications(ast);
    let ast = match_mover::move_matches(ast);
    ast
}
