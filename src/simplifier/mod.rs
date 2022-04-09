use crate::data::Exp;

mod id_dedup;
mod let_remover;

pub fn simplify<Annot>(ast: Exp<Annot>) -> Exp<Annot>
where
    Annot: Clone,
{
    let ast = let_remover::remove_lets(ast);
    let ast = id_dedup::dedup_ids(ast);
    ast
}
