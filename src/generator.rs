use crate::annotater::Annot;
use crate::data::{Direction, Exp, Machine, Node, Pat, Transition};

use std::collections::{HashMap, HashSet};

/// Generates a turing machine from an expression which evaluates to a tape -> tape function.
pub fn generate(ast: Exp<Annot>) -> Machine {
    let mut m = Machine::new();
    assert!(generate_function(&ast, &mut m, 0, 1, &HashMap::new()));
    m.simplify();
    m
}

/// Generates a turing machine from a function.
pub fn generate_function(
    ast: &Exp<Annot>,
    m: &mut Machine,
    src: usize,
    dst: usize,
    rec: &HashMap<String, usize>,
) -> bool {
    generate_set(&ast, m, src, dst)
        || generate_move(&ast, m, src, dst)
        || generate_halt(&ast, m, src)
        || generate_y(&ast, m, src, dst, rec)
        || match &ast.0 {
            Node::Function { arg, exp } => generate_from_tape(exp, &arg, m, src, dst, rec),
            Node::Identifier(id) => {
                m.push_transition(Transition {
                    from: (src, None),
                    to: (*rec.get(id).unwrap(), None),
                    dir: Direction::Stay,
                });
                true
            }
            Node::Abort => true,
            _ => false,
        }
}

/// Generates a turing machine from an expression knowing that an identifier is a tape.
fn generate_from_tape(
    ast: &Exp<Annot>,
    id: &str,
    m: &mut Machine,
    src: usize,
    dst: usize,
    rec: &HashMap<String, usize>,
) -> bool {
    match &ast.0 {
        Node::Identifier(id2) => {
            assert_eq!(id, id2);
            m.push_transition(Transition {
                from: (src, None),
                to: (dst, None),
                dir: Direction::Stay,
            });
            true
        }
        Node::Abort => true,
        _ => {
            generate_application(ast, id, m, src, dst, rec)
                || generate_match(ast, id, m, src, dst, rec)
        }
    }
}

/// Generates a turing machine from an application of a tape -> tape function.
fn generate_application(
    ast: &Exp<Annot>,
    id: &str,
    m: &mut Machine,
    src: usize,
    dst: usize,
    rec: &HashMap<String, usize>,
) -> bool {
    match &ast.0 {
        Node::Application { func, arg } => {
            let s = m.push_state();
            generate_from_tape(arg, id, m, src, s, rec) && generate_function(func, m, s, dst, rec)
        }
        _ => false,
    }
}

/// Generates a turing machine from a match expression.
fn generate_match(
    ast: &Exp<Annot>,
    id: &str,
    m: &mut Machine,
    src: usize,
    dst: usize,
    rec: &HashMap<String, usize>,
) -> bool {
    match &ast.0 {
        Node::Match { exp, arms } => {
            assert!(matches!(&exp.0, Node::Identifier(id2) if id == id2));
            let s = m.push_state();
            assert!(generate_from_tape(exp, id, m, src, s, rec));

            for arm in arms {
                let mut symbols = HashSet::new();
                match &arm.pat {
                    Pat::Union(u) => assert!(u.union_to_set(&mut symbols)),
                    _ => unreachable!(),
                };

                if !symbols.is_empty() {
                    let a = m.push_state();
                    for sym in symbols {
                        m.push_transition(Transition {
                            from: (s, Some(sym.clone())),
                            to: (a, Some(sym)),
                            dir: Direction::Stay,
                        });
                    }
                    assert!(generate_from_tape(&arm.exp, id, m, a, dst, rec));
                }
            }

            true
        }
        _ => false,
    }
}

/// Generates a turing machine from a set function.
fn generate_set(func: &Exp<Annot>, m: &mut Machine, src: usize, dst: usize) -> bool {
    if let Node::Application { func, arg } = &func.0 {
        match (&func.0, &arg.0) {
            (Node::Identifier(func), Node::Symbol(s)) if func == "set" => {
                m.push_transition(Transition {
                    from: (src, None),
                    to: (dst, Some(s.clone())),
                    dir: Direction::Stay,
                });
                true
            }
            _ => false,
        }
    } else {
        false
    }
}

/// Generates a turing machine from a next/prev function.
fn generate_move(func: &Exp<Annot>, m: &mut Machine, src: usize, dst: usize) -> bool {
    match &func.0 {
        Node::Identifier(func) if func == "prev" || func == "next" => {
            m.push_transition(Transition {
                from: (src, None),
                to: (dst, None),
                dir: if func == "prev" {
                    Direction::Left
                } else {
                    Direction::Right
                },
            });
            true
        }
        _ => false,
    }
}

/// Generates a turing machine from a halt expression.
fn generate_halt(func: &Exp<Annot>, m: &mut Machine, src: usize) -> bool {
    match &func.0 {
        Node::Identifier(func) if func == "accept" || func == "reject" => {
            m.push_transition(Transition {
                from: (src, None),
                to: (if func == "accept" { 1 } else { 2 }, None),
                dir: Direction::Stay,
            });
            true
        }
        _ => false,
    }
}

/// Generates a turing machine from a Y combinator expression.
fn generate_y(
    func: &Exp<Annot>,
    m: &mut Machine,
    src: usize,
    dst: usize,
    rec: &HashMap<String, usize>,
) -> bool {
    if let Node::Application { func, arg } = &func.0 {
        match (&func.0, &arg.0) {
            (Node::Identifier(func), Node::Function { arg: rec_id, exp }) if func == "Y" => {
                let s = m.push_state();
                m.push_transition(Transition {
                    from: (src, None),
                    to: (s, None),
                    dir: Direction::Stay,
                });

                let mut rec = rec.clone();
                rec.insert(rec_id.clone(), s);
                assert!(generate_function(exp, m, s, dst, &rec));
                true
            }
            _ => false,
        }
    } else {
        false
    }
}
