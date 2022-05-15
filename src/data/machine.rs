/// Represents a possible direction for a machine to move in.
#[derive(Debug, Copy, PartialEq, Clone)]
pub enum Direction {
    Left,
    Right,
    Stay,
}

/// Internal representation of a turing machine transition.
#[derive(Debug, Clone)]
pub struct Transition {
    pub from: (usize, Option<String>),
    pub to: (usize, Option<String>),
    pub dir: Direction,
}

/// Internal representation of a turing machine, used by the generator.
/// The indices 0, 1 and 2 are reserved for the initial, accepting and rejecting states.
#[derive(Debug, Clone)]
pub struct Machine {
    pub state_count: usize,
    pub transitions: Vec<Transition>,
}

impl Machine {
    /// Create a new machine without any transitions.
    pub fn new() -> Machine {
        Machine {
            state_count: 3,
            transitions: Vec::new(),
        }
    }

    /// Adds a new state to the machine.
    pub fn push_state(&mut self) -> usize {
        self.state_count += 1;
        self.state_count - 1
    }

    /// Adds a new transition to the machine.
    pub fn push_transition(&mut self, transition: Transition) {
        self.transitions.push(transition);
    }

    /// Simplifies the machine by removing most transitions with None symbols.
    pub fn simplify(&mut self) {
        // Join equivalent states.
        loop {
            let mut changed = false;

            for t in self.transitions.iter_mut() {
                if t.from.1.is_some() && t.to.1.is_none() {
                    t.to.1 = t.from.1.clone();
                    changed = true;
                }
            }

            if let Some((first, second)) = self.transitions.iter().find_map(|t| {
                if (&t.from.1, &t.to.1) == (&None, &None)
                    && t.dir == Direction::Stay
                    && (self.indeg(t.to.0) == 1 || self.outdeg(t.from.0) == 1)
                    && t.to.0 != 1
                    && t.to.0 != 2
                {
                    Some((t.from.0, t.to.0))
                } else {
                    None
                }
            }) {
                self.merge_states(first, second);
                changed = true;
            }

            // Search states with only one incoming and outgoing transition, and replace them by a single transition.
            if let Some(state) =
                (0..self.state_count).find(|&s| self.indeg(s) == 1 && self.outdeg(s) == 1)
            {
                // Get the transitions.
                if let Some(t) = {
                    let incoming = self.transitions.iter().find(|t| t.to.0 == state).unwrap();
                    let outgoing = self.transitions.iter().find(|t| t.from.0 == state).unwrap();
                    Self::merge_transitions(incoming.clone(), outgoing.clone())
                } {
                    self.push_transition(t);
                    self.remove_state(state);
                    changed = true;
                }
            }

            // Remove useless transitions.
            let len = self.transitions.len();
            self.transitions.retain(|t| {
                (&t.from.1, &t.to.1) != (&None, &None)
                    || t.dir != Direction::Stay
                    || t.from.0 != t.to.0
            });
            if len != self.transitions.len() {
                changed = true;
            }

            // Remove dead states.
            if let Some(state) =
                (0..self.state_count).find(|&s| self.indeg(s) == 0 && s != 0 && s != 1 && s != 2)
            {
                self.remove_state(state);
                changed = true;
            }

            if !changed {
                break;
            }
        }
    }

    /// Removes a state from the machine.
    fn remove_state(&mut self, state: usize) {
        assert!(state < self.state_count);
        self.transitions
            .retain(|t| t.from.0 != state && t.to.0 != state);
        self.state_count -= 1;
        self.transitions.iter_mut().for_each(|t| {
            if t.from.0 > state {
                t.from.0 -= 1;
            }
            if t.to.0 > state {
                t.to.0 -= 1;
            }
        });
    }

    /// Merges two states.
    fn merge_states(&mut self, lhs: usize, rhs: usize) {
        let (lhs, rhs) = if lhs > rhs {
            (rhs, lhs)
        } else if lhs == rhs {
            return;
        } else {
            (lhs, rhs)
        };

        self.transitions.iter_mut().for_each(|t| {
            if t.from.0 == rhs {
                t.from.0 = lhs;
            }
            if t.to.0 == rhs {
                t.to.0 = lhs;
            }
        });

        self.remove_state(rhs);
    }

    /// Checks the indegree of a state.
    fn indeg(&self, state: usize) -> usize {
        self.transitions.iter().filter(|t| t.to.0 == state).count()
    }

    /// Checks the outdegree of a state.
    fn outdeg(&self, state: usize) -> usize {
        self.transitions
            .iter()
            .filter(|t| t.from.0 == state)
            .count()
    }

    fn merge_transitions(incoming: Transition, outgoing: Transition) -> Option<Transition> {
        if incoming.to.0 != outgoing.from.0 {
            return None;
        }

        if incoming.dir == Direction::Stay && outgoing.dir == Direction::Stay {
            if incoming.to.1 == outgoing.from.1 || outgoing.from.1 == None {
                if incoming.to.1 == outgoing.to.1 || outgoing.to.1 == None {
                    return Some(Transition {
                        from: incoming.from,
                        to: (outgoing.to.0, incoming.to.1),
                        dir: Direction::Stay,
                    });
                } else {
                    return Some(Transition {
                        from: incoming.from,
                        to: outgoing.to,
                        dir: Direction::Stay,
                    });
                }
            }
        } else if incoming.dir == Direction::Stay {
            if incoming.to.1 == outgoing.from.1 || outgoing.from.1 == None {
                if incoming.to.1 == outgoing.to.1 || outgoing.to.1 == None {
                    return Some(Transition {
                        from: incoming.from,
                        to: (outgoing.to.0, incoming.to.1),
                        dir: outgoing.dir,
                    });
                } else {
                    return Some(Transition {
                        from: incoming.from,
                        to: outgoing.to,
                        dir: outgoing.dir,
                    });
                }
            }
        } else if outgoing.dir == Direction::Stay {
            if incoming.to.1 == outgoing.from.1 || outgoing.from.1 == None {
                if incoming.to.1 == outgoing.to.1 || outgoing.to.1 == None {
                    return Some(Transition {
                        from: incoming.from,
                        to: (outgoing.to.0, incoming.to.1),
                        dir: incoming.dir,
                    });
                }
            }
        }

        None
    }
}
