use crate::data::{Exp, TokenLoc};

mod type_checker;

pub fn annotate(exp: Exp<TokenLoc>) -> Result<Exp<type_checker::Annot>, String> {
    let ast = type_checker::type_check(exp)?;
    Ok(ast)
}
