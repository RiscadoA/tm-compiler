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

    /// Merges another machine into this one.
    pub fn merge(&mut self, other: &Machine, initial: usize, accept: usize, reject: usize) {
        assert!(initial < self.state_count);
        assert!(accept < self.state_count);
        assert!(reject < self.state_count);

        let extra_state_count = self.state_count - 3;
        self.state_count += other.state_count - 3;

        let map_index = |i: usize| {
            if i == 0 {
                initial
            } else if i == 1 {
                accept
            } else if i == 2 {
                reject
            } else {
                i + extra_state_count
            }
        };

        self.transitions
            .extend(other.transitions.iter().map(|t| Transition {
                from: (map_index(t.from.0), t.from.1.clone()),
                to: (map_index(t.to.0), t.to.1.clone()),
                dir: t.dir,
            }));
    }
}
