use crate::data::{Direction, Machine};

/// Converts a state to a string compatible with the awmorp format.
fn convert_state(state: usize) -> String {
    if state == 0 {
        "0".to_owned()
    } else if state == 1 {
        "halt-accept".to_owned()
    } else if state == 2 {
        "halt-reject".to_owned()
    } else {
        format!("{}", state - 2)
    }
}

/// Converts a symbol to a character compatible with the awmorp format.
fn convert_symbol(symbol: &Option<String>) -> Result<char, String> {
    match symbol {
        Some(s) if s.is_empty() => Ok('_'),
        Some(s) if s.len() > 1 => Err(format!(
            "Unsupported symbol '{}', only one character allowed",
            s
        )),
        Some(s) => match s.chars().next().unwrap() {
            '_' | ';' | '*' => Err(format!("Unsupported symbol '{}', reserved symbol", s)),
            c if c.is_whitespace() => Err(format!(
                "Unsupported symbol '{}', whitespace not allowed",
                s
            )),
            c => Ok(c),
        },
        None => Ok('*'),
    }
}

/// Converts a direction to a string compatible with the awmorp format.
fn convert_direction(dir: Direction) -> &'static str {
    match dir {
        Direction::Left => "l",
        Direction::Right => "r",
        Direction::Stay => "*",
    }
}

/// Exports a turing machine to the format used in the turing machine emulator
/// at https://github.com/awmorp/turing.
pub fn export(machine: Machine) -> Result<String, String> {
    let mut result = String::new();

    let mut transitions = machine.transitions;
    transitions.sort_by(|a, b| a.from.0.cmp(&b.from.0).then(a.to.0.cmp(&b.to.0)));

    for t in transitions.iter() {
        result.push_str(&format!(
            "{} {} {} {} {}\n",
            convert_state(t.from.0),
            convert_symbol(&t.from.1)?,
            convert_symbol(&t.to.1)?,
            convert_direction(t.dir),
            convert_state(t.to.0),
        ));
    }

    Ok(result)
}
