use std::collections::HashMap;

/// Represents the possible expression types.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Symbol,
    Union,
    Tape { owned: bool },
    Function { arg: Box<Type>, ret: Box<Type> },
    Unresolved(usize),
}

/// Stores the unresolved types and their possible types.
/// An unresolved type can be resolved to any of their possible types.
/// If there isn't an entry for the unresolved type, then it isn't bound to any
/// type.
#[derive(Debug)]
pub struct TypeTable {
    unresolved_count: usize,
    possibilities: HashMap<usize, Vec<Type>>,
}

impl TypeTable {
    /// Creates a new type table.
    pub fn new() -> Self {
        Self {
            unresolved_count: 0,
            constraints: HashMap::new(),
        }
    }

    /// Creates a new unresolved type.
    pub fn push(&mut self) -> Type {
        self.unresolved_count += 1;
        self.constraints
            .insert(self.unresolved_count - 1, Vec::new());
        Type::Unresolved(self.unresolved_count - 1)
    }

    /// Creates a new unresolved type with a constraint.
    pub fn push_constrained(&mut self, contraints: Vec<Type>) -> Type {
        self.unresolved_count += 1;
        self.constraints
            .insert(self.unresolved_count - 1, contraints);
        Type::Unresolved(self.unresolved_count - 1)
    }

    /// Tries to resolve a type. If the type couldn't be resolved, it is returned.
    pub fn resolve(&mut self, src: Type) -> Type {
        // If the type is already resolved, return it.
        let src = match src {
            Type::Unresolved(id) => id,
            Type::Function { arg, ret } => {
                let arg = Box::new(self.resolve(*arg));
                let ret = Box::new(self.resolve(*ret));
                return Type::Function { arg, ret };
            }
            src => return src,
        };

        // Check if the unresolved type has constraints.
        let constraints = match self.constraints.get(&src) {
            Some(constraints) => constraints,
            None => return Type::Unresolved(src),
        };

        // Try to resolve unresolved constraints.
        let mut constraints = constraints.clone();
        for c in constraints.iter_mut() {
            *c = self.resolve(c.clone());
        }

        loop {
            // If there is only one constraint, return it.
            match constraints.len() {
                0 => return Type::Unresolved(src),
                1 => return constraints[0].clone(),
                _ => (),
            };

            // Find duplicate constraints.
            let duplicates = constraints.iter().enumerate().find_map(|(i1, c1)| {
                constraints.iter().enumerate().find_map(|(i2, c2)| {
                    if c1 == c2 && i1 < i2 {
                        Some((i1, i2))
                    } else {
                        None
                    }
                })
            });

            // Remove duplicate constraint.
            if let Some((_, i2)) = duplicates {
                constraints.remove(i2);
            } else {
                break;
            }
        }

        Type::Unresolved(src)
    }

    /// Tries to cast from one type to another.
    /// If one of the types is unresolved, the necessary constraints are added.
    /// If the types are not compatible, returns false, otherwise returns true.
    pub fn cast(&mut self, from: &Type, to: &Type) -> bool {
        let from = self.resolve(from.clone());
        let to = self.resolve(to.clone());

        match (from, to) {
            (Type::Unresolved(from), Type::Unresolved(to)) => {
                // Check constraints of both types.
                match self.constraints.get(&from).map(|c| c.clone()) {
                    Some(from_c) => {
                        let to_c = self.constraints.get_mut(&to).unwrap();
                        to_c.retain(|c| from_c.contains(c));
                        if to_c.len() == 0 {
                            return false;
                        }
                    }
                    None => (),
                };

                self.constraints.insert(from, vec![Type::Unresolved(to)]);
                true
            }
            (Type::Unresolved(id), Type::Function { arg, ret }) => {
                let from_c = self.constraints.get(&id).unwrap();
                for c in from_c {}
                unimplemented!()
            }
            (Type::Unresolved(id), t) => match self.constraints.get(&id).map(|c| c.clone()) {
                Some(from_c) => {
                    let to_c = self.constraints.get_mut(&to).unwrap();
                    to_c.retain(|c| from_c.contains(c));
                    if to_c.len() == 0 {
                        return false;
                    }
                }
                None => {
                    self.constraints.insert(id, vec![t.clone()]);
                    true
                }
            },
            (to, Type::Unresolved(from)) => self.cast(&Type::Unresolved(from), &to),
            (from, to) => from == to,
        }
    }

    fn compatible(from: &Type, to: &Type) -> bool {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_easy_resolve() {
        let mut table = TypeTable::new();
        let mut t1 = table.push();
        let mut t2 = table.push();
        let mut t3 = table.push_constrained(vec![Type::Union, t2.clone()]);

        assert_eq!(table.cast(&t2, &Type::Symbol), true);
        assert_eq!(table.cast(&t1, &t3), true);
        assert_eq!(table.resolve(t1.clone()), table.resolve(t3.clone()));
        assert_eq!(table.cast(&t2, &t3), true);
        assert_eq!(table.resolve(t1.clone()), table.resolve(t3.clone()));
    }
}
