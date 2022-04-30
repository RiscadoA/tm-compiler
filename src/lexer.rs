use crate::data::{Token, TokenLoc};

use std::collections::HashMap;

struct State<'a, 'b> {
    dir: Option<&'a std::path::Path>,
    lib: &'b HashMap<String, String>,
    toks: Vec<(Token, TokenLoc)>,
    loc: TokenLoc,
    acc: String,
    in_quotes: bool,
    is_import: bool,
    is_comment: bool,
}

// All single character punctuation characters.
const PUNCTUATION: [(char, &Token); 10] = [
    ('(', &Token::LParenthesis),
    (')', &Token::RParenthesis),
    ('{', &Token::LBraces),
    ('}', &Token::RBraces),
    (':', &Token::Colon),
    ('>', &Token::Arrow),
    ('=', &Token::Assign),
    (',', &Token::Comma),
    ('|', &Token::Pipe),
    ('@', &Token::Catch),
];

// All keywords except import.
const KEYWORDS: [(&str, &Token); 4] = [
    ("match", &Token::Match),
    ("any", &Token::Any),
    ("let", &Token::Let),
    ("in", &Token::In),
];

/// Converts a string into a vector of tokens.
/// Any import expression is replaced with the contents of the file.
/// Default libraries can be added by adding them to the `libs` map, which can then be imported by their key.
pub fn tokenize(
    src: &str,
    dir: Option<&std::path::Path>,
    lib: &HashMap<String, String>,
    import_name: Option<String>,
) -> Result<Vec<(Token, TokenLoc)>, String> {
    let mut state = State::new(dir, lib, import_name);
    for chr in src.chars() {
        state.push(chr)?;
    }
    state.consume()?;
    Ok(state.toks)
}

/// Loads a string from a file and runs tokenize() on it.
pub fn tokenize_from_file(
    path: &std::path::Path,
    lib: &HashMap<String, String>,
    import_name: Option<String>,
) -> Result<Vec<(Token, TokenLoc)>, String> {
    let src = std::fs::read_to_string(path)
        .map_err(|e| format!("Couldn't tokenize file '{}': {}", path.to_str().unwrap(), e))?;
    tokenize(&src, path.parent(), lib, import_name)
}

impl<'a, 'b> State<'a, 'b> {
    // Initializes the lexer state.
    fn new(
        dir: Option<&'a std::path::Path>,
        lib: &'b HashMap<String, String>,
        import_name: Option<String>,
    ) -> Self {
        Self {
            dir,
            lib,
            toks: Vec::new(),
            loc: TokenLoc {
                line: 1,
                col: 1,
                import: import_name,
            },
            acc: String::new(),
            in_quotes: false,
            is_import: false,
            is_comment: false,
        }
    }

    // Imports a file and appends the tokens in it to the current program.
    fn import(&mut self, path: String) -> Result<(), String> {
        match self.lib.get(&path) {
            Some(src) => {
                self.toks
                    .extend(tokenize(src, None, self.lib, Some(path.clone()))?);
                Ok(())
            }
            None => {
                // Try searching for the file in the current directory.
                if let Some(dir) = &self.dir {
                    let mut p = dir.to_path_buf();
                    p.push(&path);
                    if p.exists() {
                        self.toks
                            .extend(tokenize_from_file(&p, self.lib, Some(path.clone()))?);
                        return Ok(());
                    }
                }

                Err(format!("Couldn't import file {} at {}", path, self.loc))
            }
        }
    }

    // Pushes another character for the lexer to process.
    fn push(&mut self, chr: char) -> Result<(), String> {
        if self.is_comment {
            if chr == '\n' {
                self.is_comment = false;
                self.loc.col = 1;
                self.loc.line += 1;
            }
        } else if self.in_quotes {
            if chr == '\'' {
                if self.is_import {
                    self.import(self.acc.clone())?;
                    self.is_import = false;
                } else {
                    self.push_tok(Token::Symbol(self.acc.clone()));
                }

                self.in_quotes = false;
                self.loc.col += self.acc.len() + 2;
                self.acc.clear();
            } else {
                if chr == '\n' {
                    return Err(format!(
                        "Found new line inside quotes at {}, which isn't supported",
                        self.loc
                    ));
                }
                self.acc.push(chr);
            }
        } else if chr == '\'' {
            self.consume()?;
            self.in_quotes = true;
        } else if chr == '#' {
            self.is_comment = true;
        } else if chr.is_whitespace() {
            self.consume()?;
            if chr == '\n' {
                self.loc.col = 1;
                self.loc.line += 1;
            } else {
                self.loc.col += 1;
            }
        } else if let Some(&(_, tok)) = PUNCTUATION.iter().find(|(c, _)| c == &chr) {
            self.consume()?;
            self.push_tok(tok.clone());
            self.loc.col += 1;
        } else {
            self.acc.push(chr);
        }

        Ok(())
    }

    // Consumes accumulated characters found between punctuation and whitespaces.
    fn consume(&mut self) -> Result<(), String> {
        if !self.acc.is_empty() {
            // Check if it is an import
            if self.is_import {
                return Err(format!(
                    "Expected import path after import keyword at {}, instead found '{}'",
                    self.loc, self.acc
                ));
            } else if self.acc == "import" {
                self.is_import = true;
            } else if let Some(&(_, tok)) = KEYWORDS.iter().find(|(s, _)| s == &self.acc) {
                self.push_tok(tok.clone());
            } else if is_valid_id(&self.acc) {
                self.push_tok(Token::Identifier(self.acc.clone()));
            } else {
                return Err(format!(
                    "Unknown keyword or invalid identifier '{}' found at {}",
                    self.acc, self.loc
                ));
            }
        }

        self.loc.col += self.acc.len();
        self.acc.clear();
        Ok(())
    }

    // Pushes a new token to the output.
    fn push_tok(&mut self, tok: Token) {
        self.toks.push((tok, self.loc.clone()));
    }
}

// Checks if a string is a valid identifier.
fn is_valid_id(id: &str) -> bool {
    let mut it = id.chars();
    let c = it.next().unwrap();
    id == "_" || (c.is_alphabetic() && it.all(|c| c.is_alphanumeric() || c == '_'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_empty() {
        let tokens = tokenize("", None, &HashMap::new(), None)
            .unwrap()
            .into_iter()
            .map(|t| t.0)
            .collect::<Vec<_>>();
        assert_eq!(tokens.len(), 0);
    }

    #[test]
    fn test_tokenize_import() {
        let mut lib = HashMap::new();
        lib.insert("lib".to_owned(), "'symbol3'".to_owned());
        let tokens = tokenize(
            "'symbol1' 'symbol2' import 'lib' 'symbol4'",
            None,
            &lib,
            None,
        )
        .unwrap()
        .into_iter()
        .map(|t| t.0)
        .collect::<Vec<_>>();
        assert_eq!(
            tokens,
            &[
                Token::Symbol("symbol1".to_owned()),
                Token::Symbol("symbol2".to_owned()),
                Token::Symbol("symbol3".to_owned()),
                Token::Symbol("symbol4".to_owned())
            ]
        )
    }

    #[test]
    fn test_identifiers() {
        let tokens = tokenize("_ a a_ b1 c_0", None, &HashMap::new(), None)
            .unwrap()
            .into_iter()
            .map(|t| t.0)
            .collect::<Vec<_>>();
        assert_eq!(
            tokens,
            &[
                Token::Identifier("_".to_owned()),
                Token::Identifier("a".to_owned()),
                Token::Identifier("a_".to_owned()),
                Token::Identifier("b1".to_owned()),
                Token::Identifier("c_0".to_owned())
            ]
        );

        let err = tokenize("_a", None, &HashMap::new(), None).unwrap_err();
        assert_eq!(
            err,
            "Unknown keyword or invalid identifier '_a' found at line 1, column 1"
        );

        let err = tokenize("ola\n  0teste", None, &HashMap::new(), None).unwrap_err();
        assert_eq!(
            err,
            "Unknown keyword or invalid identifier '0teste' found at line 2, column 3"
        );
    }

    #[test]
    fn test_tokenize_complex() {
        let tokens = tokenize(
            "
                        let
                            pat = 'A' | 'B',
                        in
                            (t: match get t {
                                _ @ pat > next t,
                                x @ any > set x t,
                            })
            ",
            None,
            &HashMap::new(),
            None,
        )
        .unwrap()
        .into_iter()
        .map(|t| t.0)
        .collect::<Vec<_>>();
        assert_eq!(
            tokens,
            &[
                Token::Let,
                Token::Identifier("pat".to_owned()),
                Token::Assign,
                Token::Symbol("A".to_owned()),
                Token::Pipe,
                Token::Symbol("B".to_owned()),
                Token::Comma,
                Token::In,
                Token::LParenthesis,
                Token::Identifier("t".to_owned()),
                Token::Colon,
                Token::Match,
                Token::Identifier("get".to_owned()),
                Token::Identifier("t".to_owned()),
                Token::LBraces,
                Token::Identifier("_".to_owned()),
                Token::Catch,
                Token::Identifier("pat".to_owned()),
                Token::Arrow,
                Token::Identifier("next".to_owned()),
                Token::Identifier("t".to_owned()),
                Token::Comma,
                Token::Identifier("x".to_owned()),
                Token::Catch,
                Token::Any,
                Token::Arrow,
                Token::Identifier("set".to_owned()),
                Token::Identifier("x".to_owned()),
                Token::Identifier("t".to_owned()),
                Token::Comma,
                Token::RBraces,
                Token::RParenthesis,
            ]
        )
    }
}
