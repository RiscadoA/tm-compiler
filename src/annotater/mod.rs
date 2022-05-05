use crate::data::{Exp, TokenLoc};

mod ownership_checker;
mod type_checker;
mod union_resolver;

pub use type_checker::Annot;

pub fn annotate(ast: Exp<TokenLoc>) -> Result<Exp<Annot>, String> {
    let ast = type_checker::type_check(ast)?;
    ownership_checker::ownership_check(&ast)?;
    Ok(union_resolver::resolve_unions(ast))
}
