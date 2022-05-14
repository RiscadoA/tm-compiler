/// Represents a possible direction for a machine to move in.
#[derive(Debug, Copy, Clone)]
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
}
