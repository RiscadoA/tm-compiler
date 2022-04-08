use crate::data::{Arm, Exp, Node, Pat, Token, TokenLoc};

/// Input accepted by parse_x functions.
type Stream<'a> = &'a [(Token, TokenLoc)];
/// Result returned by parse_x functions.
type Result<'a> = std::result::Result<Option<(Stream<'a>, Exp<TokenLoc>)>, String>;

/// Converts a stream of tokens into an abstract syntax tree.
pub fn parse(toks: Vec<(Token, TokenLoc)>) -> std::result::Result<Exp<TokenLoc>, String> {
    let (toks, exp) = parse_exp(&toks)?.ok_or(format!(
        "Expected expression but found {} (while parsing root expression)",
        match toks.get(0) {
            Some(t) => format!("{}", t.0),
            None => "EOF".to_owned(),
        },
    ))?;

    if toks.is_empty() {
        Ok(exp)
    } else {
        Err(format!(
            "Expected EOF but found token {} at {}",
            toks[0].0, toks[0].1
        ))
    }
}

fn parse_exp(toks: Stream) -> Result {
    let (mut toks, mut exp) = if let Some((toks, exp)) = parse_apply(toks)? {
        (toks, exp)
    } else {
        return Ok(None);
    };

    while let Some((t, loc)) = accept_token(toks, Token::Pipe) {
        let (t, exp_rhs) = parse_exp(t)?.ok_or(format!(
            "Expected expression but found {} after {} (while parsing union)",
            match toks.get(0) {
                Some(t) => format!("{}", t.0),
                None => "EOF".to_owned(),
            },
            loc,
        ))?;

        exp = Exp(
            Node::Union {
                lhs: Box::new(exp),
                rhs: Box::new(exp_rhs),
            },
            loc,
        );
        toks = t;
    }

    Ok(Some((toks, exp)))
}

/// Parses an apply operation.
fn parse_apply(toks: Stream) -> Result {
    let (mut toks, mut exp) = if let Some((toks, exp)) = parse_term(toks)? {
        (toks, exp)
    } else {
        return Ok(None);
    };

    while let Some((t, arg)) = parse_term(toks)? {
        exp = Exp(
            Node::Application {
                func: Box::new(exp.clone()),
                arg: Box::new(arg),
            },
            exp.1,
        );
        toks = t;
    }

    Ok(Some((toks, exp)))
}

/// Parses a term.
fn parse_term(toks: Stream) -> Result {
    if let Some((toks, loc)) = accept_token(toks, Token::LParenthesis) {
        let (toks, exp) = match parse_exp(toks)? {
            Some(r) => r,
            None => {
                return Err(format!(
                "Expected expression but found {} after {} (while parsing parenthesis expression)",
                match toks.get(0) {
                    Some(t) => format!("{}", t.0),
                    None => "EOF".to_owned(),
                },
                loc,
            ))
            }
        };

        let (toks, _) = expect_token(
            toks,
            Token::RParenthesis,
            "while parsing parenthesis expression",
        )?;

        Ok(Some((toks, exp)))
    } else if let Some((toks, exp)) = parse_match(toks)? {
        Ok(Some((toks, exp)))
    } else if let Some((toks, exp)) = parse_let(toks)? {
        Ok(Some((toks, exp)))
    } else if let Some((toks, exp)) = parse_function(toks)? {
        Ok(Some((toks, exp)))
    } else if let Some((toks, exp)) = parse_identifier(toks)? {
        Ok(Some((toks, exp)))
    } else if let Some((toks, exp)) = parse_symbol(toks)? {
        Ok(Some((toks, exp)))
    } else {
        Ok(None)
    }
}

/// Parses a match.
fn parse_match(toks: Stream) -> Result {
    let (toks, loc) = match accept_token(toks, Token::Match) {
        Some((toks, loc)) => (toks, loc),
        None => return Ok(None),
    };

    let (toks, exp) = parse_exp(toks)?.ok_or(format!(
        "Expected expression but found {} after {} (while parsing match expression)",
        match toks.get(0) {
            Some(t) => format!("{}", t.0),
            None => "EOF".to_owned(),
        },
        loc,
    ))?;

    let (mut toks, mut last_loc) =
        expect_token(toks, Token::LBraces, "while parsing match expression")?;

    let mut arms = Vec::new();
    let (toks, _) = loop {
        if let Some(t) = accept_token(toks, Token::RBraces) {
            break t;
        }

        let catch_id = if let Some((t, id, _)) = accept_identifier(toks) {
            if let Some((t, loc)) = accept_token(t, Token::Catch) {
                toks = t;
                last_loc = loc;
                Some(id)
            } else {
                None
            }
        } else {
            None
        };

        let (t, pat) = if let Some((t, _)) = accept_token(toks, Token::Any) {
            (t, Pat::Any)
        } else {
            let (t, exp) = parse_exp(toks)?.ok_or(format!(
                "Expected expression but found {} after {} (while parsing match pattern)",
                match toks.get(0) {
                    Some(t) => format!("{}", t.0),
                    None => "EOF".to_owned(),
                },
                last_loc,
            ))?;
            (t, Pat::Union(exp))
        };

        let (t, loc) = expect_token(t, Token::Arrow, "while parsing match arm")?;
        let (t, exp) = parse_exp(t)?.ok_or(format!(
            "Expected expression but found {} after {} (while parsing match arm)",
            match toks.get(0) {
                Some(t) => format!("{}", t.0),
                None => "EOF".to_owned(),
            },
            loc,
        ))?;

        let (t, loc) = expect_token(t, Token::Comma, "while parsing match arm")?;
        toks = t;
        last_loc = loc;

        arms.push(Arm { catch_id, pat, exp });
    };

    let exp = Box::new(exp);
    Ok(Some((toks, Exp(Node::Match { exp, arms }, loc))))
}

/// Parses a let expression.
fn parse_let(toks: Stream) -> Result {
    let (mut toks, loc) = match accept_token(toks, Token::Let) {
        Some((toks, loc)) => (toks, loc),
        None => return Ok(None),
    };

    let mut binds = Vec::new();
    let (toks, last_loc) = loop {
        if let Some(t) = accept_token(toks, Token::In) {
            break t;
        }

        let (t, id, _) = expect_identifier(toks, "while parsing let expression")?;
        let (t, loc) = expect_token(t, Token::Assign, "while parsing let binding")?;
        let (t, exp) = parse_exp(t)?.ok_or(format!(
            "Expected expression but found {} after {} (while parsing let binding)",
            match t.get(0) {
                Some(t) => format!("{}", t.0),
                None => "EOF".to_owned(),
            },
            loc,
        ))?;

        let (t, _) = expect_token(t, Token::Comma, "while parsing let expression")?;
        toks = t;
        binds.push((id, exp))
    };

    let (toks, exp) = parse_exp(toks)?.ok_or(format!(
        "Expected expression after {} (while parsing let expression)",
        last_loc
    ))?;
    let exp = Box::new(exp);

    Ok(Some((toks, Exp(Node::Let { exp, binds }, loc))))
}

/// Parses a function.
fn parse_function(toks: Stream) -> Result {
    let (toks, arg, loc) = match accept_identifier(toks) {
        Some((toks, id, loc)) => (toks, id, loc),
        None => return Ok(None),
    };

    let (toks, _) = match accept_token(toks, Token::Colon) {
        Some(toks) => toks,
        None => return Ok(None),
    };

    match parse_exp(toks)? {
        Some((toks, exp)) => Ok(Some((
            toks,
            Exp(
                Node::Function {
                    arg,
                    exp: Box::new(exp),
                },
                loc,
            ),
        ))),
        None => Err(format!(
            "Expected expression but found {} at {} (while parsing function body at {})",
            toks[0].0, toks[0].1, loc
        )),
    }
}

/// Parses an identifier.
fn parse_identifier(toks: Stream) -> Result {
    Ok(if let Some((toks, id, loc)) = accept_identifier(toks) {
        Some((toks, Exp(Node::Identifier(id.clone()), loc.clone())))
    } else {
        None
    })
}

/// Parses a symbol.
fn parse_symbol(toks: Stream) -> Result {
    Ok(match toks.split_first() {
        Some(((Token::Symbol(sym), loc), rem)) => {
            Some((rem, Exp(Node::Symbol(sym.clone()), loc.clone())))
        }
        _ => None,
    })
}

/// Checks if the first token in the stream is an identifier, and if it is, returns the rest of the stream.
/// Otherwise, returns None.
fn accept_identifier(toks: Stream) -> Option<(Stream, String, TokenLoc)> {
    match toks.split_first() {
        Some(((Token::Identifier(id), loc), rem)) => Some((rem, id.clone(), loc.clone())),
        _ => None,
    }
}

/// Checks if the first token in the stream is the expected token, and if it is, returns the rest of the stream.
/// Otherwise, returns None.
fn accept_token(toks: Stream, tok: Token) -> Option<(Stream, TokenLoc)> {
    let (first, rem) = toks.split_first()?;
    if first.0 == tok {
        Some((rem, first.1.clone()))
    } else {
        None
    }
}

/// Checks if the first token in the stream is an identifier, and if it is, returns the rest of the stream.
/// Otherwise, returns an error message.
fn expect_identifier<'a>(
    toks: Stream<'a>,
    ctx: &str,
) -> std::result::Result<(Stream<'a>, String, TokenLoc), String> {
    let (first, toks) = toks
        .split_first()
        .ok_or(format!("Expected identifier but found EOF ({})", ctx))?;
    if let Token::Identifier(id) = &first.0 {
        Ok((toks, id.clone(), first.1.clone()))
    } else {
        Err(format!(
            "Expected identifier but found {} at {} ({})",
            first.0, first.1, ctx
        ))
    }
}

/// Expects that the first token in the stream is the expected token. If it is, returns the rest of the stream.
/// Otherwise, returns an error message.
fn expect_token<'a>(
    toks: Stream<'a>,
    tok: Token,
    ctx: &str,
) -> std::result::Result<(Stream<'a>, TokenLoc), String> {
    let (first, toks) = toks
        .split_first()
        .ok_or(format!("Expected token {} but found EOF ({})", tok, ctx))?;
    if first.0 == tok {
        Ok((toks, first.1.clone()))
    } else {
        Err(format!(
            "Expected token {} but found {} at {} ({})",
            tok, first.0, first.1, ctx
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let tokens = Vec::new();
        let _ast = parse(tokens)
            .expect_err("Expected expression but found EOF (while parsing root expression)");
    }

    #[test]
    fn test_parse_precedence() {
        let (tokens, dummy) = insert_dummy_locs(vec![
            Token::LParenthesis,
            Token::Identifier("x".to_owned()),
            Token::Identifier("y".to_owned()),
            Token::Pipe,
            Token::Identifier("a".to_owned()),
            Token::Identifier("b".to_owned()),
            Token::RParenthesis,
            Token::Identifier("c".to_owned()),
        ]);

        let ast = parse(tokens).unwrap();
        assert_eq!(
            ast,
            Exp(
                Node::Application {
                    func: Box::new(Exp(
                        Node::Union {
                            lhs: Box::new(Exp(
                                Node::Application {
                                    func: Box::new(Exp(
                                        Node::Identifier("x".to_owned()),
                                        dummy.clone()
                                    )),
                                    arg: Box::new(Exp(
                                        Node::Identifier("y".to_owned()),
                                        dummy.clone()
                                    )),
                                },
                                dummy.clone(),
                            )),
                            rhs: Box::new(Exp(
                                Node::Application {
                                    func: Box::new(Exp(
                                        Node::Identifier("a".to_owned()),
                                        dummy.clone()
                                    )),
                                    arg: Box::new(Exp(
                                        Node::Identifier("b".to_owned()),
                                        dummy.clone()
                                    )),
                                },
                                dummy.clone(),
                            )),
                        },
                        dummy.clone(),
                    )),
                    arg: Box::new(Exp(Node::Identifier("c".to_owned()), dummy.clone())),
                },
                dummy,
            )
        );
    }

    fn insert_dummy_locs(toks: Vec<Token>) -> (Vec<(Token, TokenLoc)>, TokenLoc) {
        let dummy = TokenLoc {
            line: 0,
            col: 0,
            import: None,
        };
        let toks = toks
            .into_iter()
            .map(|t| (t, dummy.clone()))
            .collect::<Vec<_>>();
        (toks, dummy)
    }
}
