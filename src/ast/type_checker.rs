use crate::data::{Arm, Exp, Node, TokenLoc, Type};

struct State {}

/// Annotates an AST with types, checking for type errors.
/// All remaining unresolved types
pub fn type_check(ast: Exp<TokenLoc>) -> Result<Exp<(Type, TokenLoc)>, String> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_checker_simple() {
        let dummy_loc = TokenLoc {
            line: 0,
            col: 0,
            import: None,
        };

        let ast = Exp(
            Node::Function {
                arg: "x".to_owned(),
                exp: Box::new(Exp(Node::Identifier("x".to_owned()), dummy_loc.clone())),
            },
            dummy_loc,
        );

        let tast = type_check(ast).unwrap();

        assert_eq!(
            tast.1 .0,
            Type::Function {
                arg: Box::new(Type::Unresolved(0)),
                ret: Box::new(Type::Unresolved(0)),
            }
        );
    }
}
