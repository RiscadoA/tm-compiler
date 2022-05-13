use super::TokenLoc;
use std::collections::HashMap;
use std::fmt;

/// Represents the possible expression types.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Symbol,
    Union,
    Tape,
    Halt,
    Function { arg: Box<Type>, ret: Box<Type> },
    Unresolved(usize),
    UnresolvedUnion(usize),
}

/// Stores all of 'Unresolved' types and their corresponding resolved types, if any.
#[derive(Debug)]
pub struct TypeTable {
    unresolved_count: usize,
    resolved: HashMap<usize, Type>,
}

impl Type {
    /// Returns true if the type contains an Unresolved type (UnresolvedUnion is not considered).
    pub fn is_unresolved_non_union(&self) -> bool {
        match self {
            Type::Function { arg, ret } => {
                arg.is_unresolved_non_union() || ret.is_unresolved_non_union()
            }
            Type::Unresolved(_) => true,
            _ => false,
        }
    }

    /// Returns true if the type contains an UnresolvedUnion type.
    pub fn is_unresolved_union(&self) -> bool {
        match self {
            Type::Function { arg, ret } => arg.is_unresolved_union() || ret.is_unresolved_union(),
            Type::UnresolvedUnion(_) => true,
            _ => false,
        }
    }

    /// Checks if this type can be casted to the given type. If any of the types are unresolved, true is rturned.
    pub fn simple_cast(&self, to: &Type) -> bool {
        match (self, to) {
            (
                Type::Function { arg, ret },
                Type::Function {
                    arg: arg2,
                    ret: ret2,
                },
            ) => arg2.simple_cast(arg) && ret.simple_cast(ret2),
            (Type::Halt, Type::Function { .. }) => false,
            (Type::Function { .. }, Type::Halt) => false,
            (Type::Halt, _) => true,
            (_, Type::Halt) => true,
            (Type::Symbol, Type::Union) => true,
            (Type::UnresolvedUnion(_), Type::Symbol) => true,
            (Type::UnresolvedUnion(_), Type::Union) => true,
            (Type::Symbol, Type::UnresolvedUnion(_)) => true,
            (Type::Union, Type::UnresolvedUnion(_)) => true,
            (Type::Unresolved(_), _) => true,
            (_, Type::Unresolved(_)) => true,
            (from, to) => from == to,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Type::Symbol => write!(f, "symbol"),
            Type::Union => write!(f, "union"),
            Type::Tape => write!(f, "tape"),
            Type::Function { arg, ret } => write!(f, "({} -> {})", arg, ret),
            Type::Halt => write!(f, "halt"),
            Type::Unresolved(id) => write!(f, "u{}", id),
            Type::UnresolvedUnion(_) => write!(f, "symbol/union"),
        }
    }
}

impl TypeTable {
    /// Creates a new type table.
    pub fn new() -> Self {
        Self {
            unresolved_count: 0,
            resolved: HashMap::new(),
        }
    }

    /// Creates a new unresolved type.
    pub fn push(&mut self) -> Type {
        self.unresolved_count += 1;
        Type::Unresolved(self.unresolved_count - 1)
    }

    /// Casts the given type to the given type.
    /// Returns an error if the types cannot be casted.
    pub fn cast(&mut self, from: &Type, to: &Type, loc: &TokenLoc) -> Result<(), String> {
        let from = self.resolve(from);
        let to = self.resolve(to);

        match (from, to) {
            (from, to) if from == to => {}

            (
                Type::Function { arg, ret },
                Type::Function {
                    arg: arg_to,
                    ret: ret_to,
                },
            ) => {
                self.cast(&arg_to, &arg, loc)?;
                self.cast(&ret, &ret_to, loc)?;
            }

            (from @ Type::Function { .. }, Type::Unresolved(to)) => {
                let func = Type::Function {
                    arg: Box::new(self.push()),
                    ret: Box::new(self.push()),
                };
                self.cast(&from, &func, loc)?;
                assert!(self.resolved.insert(to, func).is_none());
            }

            (Type::Unresolved(from), to @ Type::Function { .. }) => {
                let func = Type::Function {
                    arg: Box::new(self.push()),
                    ret: Box::new(self.push()),
                };
                self.cast(&func, &to, loc)?;
                assert!(self.resolved.insert(from, func).is_none());
            }

            (Type::Symbol, Type::Unresolved(id)) | (Type::Unresolved(id), Type::Union) => {
                assert!(self.resolved.insert(id, Type::UnresolvedUnion(0)).is_none());
            }

            (Type::Halt, Type::Unresolved(_)) | (Type::Unresolved(_), Type::Halt) => (),

            (t, Type::Unresolved(id)) | (Type::Unresolved(id), t) => {
                assert!(self.resolved.insert(id, t).is_none());
            }

            (from, to) => {
                if !from.simple_cast(&to) {
                    return Err(format!("Cannot cast {} to {} at {}", from, to, loc));
                }
            }
        }

        Ok(())
    }

    /// Resolves a type. If the type can't be resolved, it will be returned as is.
    pub fn resolve(&mut self, t: &Type) -> Type {
        match t {
            Type::Function { arg, ret } => Type::Function {
                arg: Box::new(self.resolve(arg)),
                ret: Box::new(self.resolve(ret)),
            },
            Type::Unresolved(id) => {
                if let Some(resolved) = self.resolved.get(id).map(|t| t.clone()) {
                    let resolved = self.resolve(&resolved);
                    self.resolved.insert(*id, resolved.clone());
                    resolved
                } else {
                    t.clone()
                }
            }
            _ => t.clone(),
        }
    }
}
