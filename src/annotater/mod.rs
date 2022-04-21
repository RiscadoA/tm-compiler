use crate::data::{Exp, TokenLoc};

mod borrow_checker;
mod type_checker;

pub fn annotate(exp: Exp<TokenLoc>) -> Result<Exp<type_checker::Annot>, String> {
    let ast = type_checker::type_check(exp)?;
    borrow_checker::borrow_check(&ast)?;
    Ok(ast)
}
