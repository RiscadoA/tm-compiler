use super::token::TokenLoc;

pub trait AnnotLoc {
    fn loc(&self) -> &TokenLoc;
}

impl AnnotLoc for TokenLoc {
    fn loc(&self) -> &TokenLoc {
        self
    }
}
