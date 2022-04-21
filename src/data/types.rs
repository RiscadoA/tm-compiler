use super::TokenLoc;
use std::collections::HashMap;
use std::fmt;

/// Represents the possible expression types.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Symbol,
    Union,
    Tape { owned: bool },
    Function { arg: Box<Type>, ret: Box<Type> },
    Unresolved(usize),
}

/// Represents a graph with all casts between types.
#[derive(Debug)]
pub struct TypeGraph {
    unresolved_count: usize,
    casts: Vec<(Type, Type, TokenLoc)>,
    resolved: HashMap<usize, Type>,
}

impl Type {
    pub fn is_resolved(&self) -> bool {
        match self {
            Type::Function { arg, ret } => arg.is_resolved() && ret.is_resolved(),
            Type::Unresolved(_) => false,
            _ => true,
        }
    }

    /// Checks if this type can be casted to the given type. Both types must be resolved.
    pub fn can_cast(&self, to: &Type) -> bool {
        match (self, to) {
            (
                Type::Function { arg, ret },
                Type::Function {
                    arg: arg2,
                    ret: ret2,
                },
            ) => arg.can_cast(arg2) && ret.can_cast(ret2),
            (Type::Tape { owned: true }, Type::Tape { owned: false }) => true,
            (Type::Symbol, Type::Union) => true,
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
            Type::Tape { owned } => write!(f, "{}tape", if *owned { "" } else { "&" }),
            Type::Function { arg, ret } => write!(f, "({} -> {})", arg, ret),
            Type::Unresolved(id) => write!(f, "unresolved{}", id),
        }
    }
}

impl TypeGraph {
    /// Creates a new type graph.
    pub fn new() -> Self {
        Self {
            unresolved_count: 0,
            casts: Vec::new(),
            resolved: HashMap::new(),
        }
    }

    /// Creates a new unresolved type.
    pub fn push(&mut self) -> Type {
        self.unresolved_count += 1;
        Type::Unresolved(self.unresolved_count - 1)
    }

    /// Registers a cast between two types.
    pub fn cast(&mut self, from: &Type, to: &Type, loc: &TokenLoc) {
        self.casts.push((from.clone(), to.clone(), loc.clone()));
    }

    /// Resolves the type graph, tries to resolve all unresolved types.
    pub fn resolve(&mut self) -> Result<(), String> {
        self.remove_function_casts();
        self.remove_cycles();

        // Find all unresolved types that cast to a resolved type and change the casts to them to their specific casts.
        loop {
            let mut changed = false;

            for i in 0..self.unresolved_count {
                // Skip already resolved types.
                if self.resolved.contains_key(&i) {
                    continue;
                }

                let mut unresolved_from = false;
                let mut unresolved_to = false;

                // All resolved types that cast to this unresolved type.
                let mut from = self
                    .casts
                    .iter()
                    .filter(|(_, to, _)| to == &Type::Unresolved(i))
                    .map(|(from, _, loc)| (self.get(&from), loc))
                    .filter(|(from, _)| {
                        if from.is_resolved() {
                            true
                        } else {
                            unresolved_from = true;
                            false
                        }
                    })
                    .collect::<Vec<_>>();

                // All resolved types which this unresolved type casts to.
                let mut to = self
                    .casts
                    .iter()
                    .filter(|(from, _, _)| from == &Type::Unresolved(i))
                    .map(|(_, to, loc)| (self.get(&to), loc))
                    .filter(|(to, _)| {
                        if to.is_resolved() {
                            true
                        } else {
                            unresolved_to = true;
                            false
                        }
                    })
                    .collect::<Vec<_>>();

                // Remove duplicates.
                for i in (1..from.len()).rev() {
                    if from[0..i].iter().any(|j| &from[i].0 == &j.0) {
                        from.remove(i);
                    }
                }

                for i in (1..to.len()).rev() {
                    if to[0..i].iter().any(|j| &to[i].0 == &j.0) {
                        to.remove(i);
                    }
                }

                // Check if all 'from' types can be combined into a single one.
                let from = if from.len() == 0 {
                    None
                } else if from.len() == 1 {
                    match &from[0].0 {
                        Type::Symbol if unresolved_from => None,
                        Type::Tape { owned: true } if unresolved_from => None,
                        f => Some((f.clone(), from[0].1)),
                    }
                } else {
                    // Check if types are compatible.
                    for i in 0..from.len() {
                        for j in (i + 1)..from.len() {
                            if !from[i].0.can_cast(&from[j].0) && !from[j].0.can_cast(&from[i].0) {
                                return Err(format!(
                                    "Cannot resolve type since incompatible types {} and {} cast to it at {} and {}",
                                    from[i].0, from[j].0, from[i].1, from[j].1
                                ));
                            }
                        }
                    }

                    if from.len() == 2 {
                        match (&from[0].0, &from[1].0) {
                            (Type::Tape { .. }, Type::Tape { .. }) => {
                                Some((Type::Tape { owned: false }, from[0].1))
                            }
                            (Type::Symbol, Type::Union) | (Type::Union, Type::Symbol) => {
                                Some((Type::Union, from[0].1))
                            }
                            _ => None,
                        }
                    } else {
                        // There must be a type which is not possible to cast.
                        unreachable!();
                    }
                };

                // Check if all 'to' types can be combined into a single one.
                let to = if to.len() == 0 {
                    None
                } else if to.len() == 1 {
                    match &to[0].0 {
                        Type::Union if unresolved_to => None,
                        Type::Tape { owned: false } if unresolved_to => None,
                        t => Some((t.clone(), to[0].1)),
                    }
                } else {
                    // There must be a type which is not possible to cast.
                    for i in 0..to.len() {
                        for j in (i + 1)..to.len() {
                            if !to[i].0.can_cast(&to[j].0) && !to[j].0.can_cast(&to[i].0) {
                                return Err(format!(
                                    "Cannot resolve type since it casts to incompatible types {} and {} at {} and {}",
                                    to[i].0, to[j].0, to[i].1, to[j].1
                                ));
                            }
                        }
                    }

                    if to.len() == 2 {
                        match (&to[0].0, &to[1].0) {
                            (Type::Tape { .. }, Type::Tape { .. }) => {
                                Some((Type::Tape { owned: true }, to[0].1))
                            }
                            (Type::Symbol, Type::Union) | (Type::Union, Type::Symbol) => {
                                Some((Type::Symbol, to[0].1))
                            }
                            _ => None,
                        }
                    } else {
                        // There must be a type which is not possible to cast.
                        unreachable!();
                    }
                };

                // Check if its possible to resolve this type.
                let t = match (&from, &to) {
                    (Some((f, _)), Some((t, tloc))) => {
                        if f.can_cast(&t) {
                            Some(f)
                        } else {
                            return Err(format!("Cannot cast {} to {} at {}", f, t, tloc));
                        }
                    }
                    (Some((Type::Tape { owned: true }, _)), None) => None,
                    (Some((Type::Symbol, _)), None) => None,
                    (Some((from, _)), None) => Some(from),
                    (None, Some((to, _))) => Some(to),
                    (None, None) => None,
                };

                if let Some(t) = t {
                    assert!(self.resolved.insert(i, t.clone()).is_none());
                    changed = true;
                }
            }

            if !changed {
                break;
            }
        }

        self.remove_resolved_casts()?;

        Ok(())
    }

    /// Gets the resolved type of a type. If the type can't be resolved, returns the type itself.
    pub fn get(&self, typ: &Type) -> Type {
        match typ {
            Type::Unresolved(id) => self
                .resolved
                .get(id)
                .map(|t| self.get(t))
                .unwrap_or(Type::Unresolved(*id)),
            Type::Function { arg, ret } => {
                let arg = self.get(arg);
                let ret = self.get(ret);
                return Type::Function {
                    arg: Box::new(arg),
                    ret: Box::new(ret),
                };
            }
            _ => typ.clone(),
        }
    }

    /// Removes Resolved -> Resolved casts.
    fn remove_resolved_casts(&mut self) -> Result<(), String> {
        // Check casts
        for (from, to, loc) in self.casts.iter() {
            let from = self.get(&from);
            let to = self.get(&to);
            if !from.can_cast(&to) {
                return Err(format!("Cannot cast {} to {} at {}", from, to, loc));
            }
        }

        // Remove Resolved -> Resolved casts.
        self.casts
            .retain(|(from, to, _)| !from.is_resolved() || !to.is_resolved());
        Ok(())
    }

    /// Removes function casts from the type graph.
    fn remove_function_casts(&mut self) {
        let mut new_casts = Vec::new();

        loop {
            for i in 0..self.casts.len() {
                self.casts[i].0 = self.get(&self.casts[i].0);
                self.casts[i].1 = self.get(&self.casts[i].1);
            }

            self.casts.retain(|(from, to, loc)| {
                let (arg1, ret1) = if let Type::Function { arg, ret } = from {
                    (*arg.clone(), *ret.clone())
                } else if let Type::Unresolved(id) = from {
                    if let Type::Function { .. } = to {
                        self.unresolved_count += 2;
                        let arg = Type::Unresolved(self.unresolved_count - 2);
                        let ret = Type::Unresolved(self.unresolved_count - 1);
                        let func = Type::Function {
                            arg: Box::new(arg.clone()),
                            ret: Box::new(ret.clone()),
                        };
                        if let Some(f) = self.resolved.insert(*id, func.clone()) {
                            new_casts.push((func, f, loc.clone()));
                        }
                        (arg, ret)
                    } else {
                        return true;
                    }
                } else {
                    return true;
                };

                let (arg2, ret2) = if let Type::Function { arg, ret } = to {
                    (*arg.clone(), *ret.clone())
                } else if let Type::Unresolved(id) = to {
                    self.unresolved_count += 2;
                    let arg = Type::Unresolved(self.unresolved_count - 2);
                    let ret = Type::Unresolved(self.unresolved_count - 1);
                    let func = Type::Function {
                        arg: Box::new(arg.clone()),
                        ret: Box::new(ret.clone()),
                    };
                    if let Some(f) = self.resolved.insert(*id, func.clone()) {
                        new_casts.push((func, f, loc.clone()));
                    }
                    (arg, ret)
                } else {
                    return true;
                };

                new_casts.push((arg1.clone(), arg2.clone(), loc.clone()));
                new_casts.push((arg2, arg1, loc.clone()));
                new_casts.push((ret1.clone(), ret2.clone(), loc.clone()));
                new_casts.push((ret2, ret1, loc.clone()));
                false
            });

            if new_casts.is_empty() {
                break;
            } else {
                self.casts.append(&mut new_casts);
            }
        }
    }

    /// Removes unresolved cycles from the graph using Tarjan's algorithm.
    fn remove_cycles(&mut self) {
        // Utility representation of unresolved types.
        struct Vertex {
            index: usize,
            lowlink: usize,
            onstack: bool,
            edges: Vec<usize>,
        }

        // Depth first search for finding strongly connected components
        fn dfs(
            verts: &mut [Vertex],
            v: usize,
            stack: &mut Vec<usize>,
            sccs: &mut Vec<Vec<usize>>,
            index: &mut usize,
        ) {
            verts[v].index = *index;
            verts[v].lowlink = *index;
            verts[v].onstack = true;
            stack.push(v);
            *index += 1;

            // Recurse.
            for i in 0..verts[v].edges.len() {
                let w = verts[v].edges[i];
                if verts[w].index == 0 {
                    dfs(verts, w, stack, sccs, index);
                    verts[v].lowlink = verts[v].lowlink.min(verts[w].lowlink);
                } else if verts[w].onstack {
                    verts[v].lowlink = verts[v].lowlink.min(verts[w].index);
                }
            }

            // Check if its an SCC root
            if verts[v].lowlink == verts[v].index {
                let mut scc = Vec::new();
                loop {
                    let w = stack.pop().unwrap();
                    verts[w].onstack = false;
                    scc.push(w);
                    if w == v {
                        break;
                    }
                }
                sccs.push(scc);
            }
        }

        // Get all vertices and their edges.
        let mut verts = Vec::new();
        for i in 0..self.unresolved_count {
            let edges = self
                .casts
                .iter()
                .filter_map(|(from, to, _)| {
                    if from == &Type::Unresolved(i) {
                        if let Type::Unresolved(j) = to {
                            if self.resolved.get(j).is_none() {
                                return Some(*j);
                            }
                        }
                    }

                    None
                })
                .collect();

            verts.push(Vertex {
                index: 0,
                lowlink: 0,
                onstack: false,
                edges,
            });
        }

        // Find strongly connected components
        let mut stack = Vec::new();
        let mut sccs = Vec::new();
        let mut index = 1;
        for v in 0..verts.len() {
            if verts[v].index == 0 && self.resolved.get(&v).is_none() {
                dfs(&mut verts, v, &mut stack, &mut sccs, &mut index);
            }
        }

        // Merge all strongly connected components into a single unresolved type.
        for scc in sccs {
            // Remove internal component casts
            self.casts.retain(|(from, to, _)| {
                if let Type::Unresolved(from) = from {
                    if let Type::Unresolved(to) = to {
                        if scc.contains(from) && scc.contains(to) {
                            return false;
                        }
                    }
                }

                true
            });

            // Choose the root type
            let root = scc[0];
            // Rename all casts from/to the SCC to use the SCC root type
            self.casts.iter_mut().for_each(|(from, to, _)| {
                if let Type::Unresolved(f) = from {
                    if scc.contains(f) {
                        *from = Type::Unresolved(root);
                    }
                }

                if let Type::Unresolved(t) = to {
                    if scc.contains(t) {
                        *to = Type::Unresolved(root);
                    }
                }
            });

            // Resolve non-root types to the root type
            for typ in scc.iter().skip(1) {
                assert!(self.resolved.insert(*typ, Type::Unresolved(root)).is_none());
            }
        }
    }
}
