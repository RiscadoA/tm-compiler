use crate::data::{Token, TokenLoc};

use std::collections::HashMap;

struct State<'a, 'b> {
    dir: Option<&'a std::path::Path>,
    lib: &'b HashMap<String, String>,
    import_name: Option<String>,
    toks: Vec<Token>,
    toks_loc: Vec<TokenLoc>,
    acc: String,
    in_quotes: bool,
    is_import: bool,
    line: usize,
    col: usize,
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
) -> Result<(Vec<Token>, Vec<TokenLoc>), String> {
    let mut state = State::new(dir, lib, import_name);
    for chr in src.chars() {
        state.push(chr)?;
    }
    state.consume()?;
    Ok((state.toks, state.toks_loc))
}

/// Loads a string from a file and runs tokenize() on it.
pub fn tokenize_from_file(
    path: &std::path::Path,
    lib: &HashMap<String, String>,
    import_name: Option<String>,
) -> Result<(Vec<Token>, Vec<TokenLoc>), String> {
    let src = std::fs::read_to_string(path).unwrap();
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
            import_name,
            toks: Vec::new(),
            toks_loc: Vec::new(),
            acc: String::new(),
            in_quotes: false,
            is_import: false,
            line: 1,
            col: 1,
        }
    }

    // Imports a file and appends the tokens in it to the current program.
    fn import(&mut self, path: String) -> Result<(), String> {
        match self.lib.get(&path) {
            Some(src) => {
                let res = tokenize(src, None, self.lib, Some(path.clone()))?;
                self.toks.extend(res.0);
                self.toks_loc.extend(res.1);
                Ok(())
            }
            None => {
                // Try searching for the file in the current directory.
                if let Some(dir) = &self.dir {
                    let mut p = dir.to_path_buf();
                    p.push(&path);
                    if p.exists() {
                        let res = tokenize_from_file(&p, self.lib, Some(path.clone()))?;
                        self.toks.extend(res.0);
                        self.toks_loc.extend(res.1);
                        return Ok(());
                    }
                }

                Err(format!(
                    "Couldn't import file {} at line {} and column {}",
                    path, self.line, self.col
                ))
            }
        }
    }

    // Pushes another character for the lexer to process.
    fn push(&mut self, chr: char) -> Result<(), String> {
        if self.in_quotes {
            if chr == '\'' {
                if self.is_import {
                    self.import(self.acc.clone())?;
                    self.is_import = false;
                } else {
                    self.push_tok(Token::Symbol(self.acc.clone()));
                }

                self.in_quotes = false;
                self.col += self.acc.len();
                self.acc.clear();
            } else {
                if chr == '\n' {
                    return Err(format!(
                        "Found new line inside quotes at line {}, which isn't supported",
                        self.line
                    ));
                }
                self.acc.push(chr);
            }
        } else if chr == '\'' {
            self.consume()?;
            self.col += 1;
            self.in_quotes = true;
        } else if chr.is_whitespace() {
            self.consume()?;
            if chr == '\n' {
                self.col = 1;
                self.line += 1;
            } else {
                self.col += 1;
            }
        } else if let Some(&(_, tok)) = PUNCTUATION.iter().find(|(c, _)| c == &chr) {
            self.consume()?;
            self.push_tok(tok.clone());
            self.col = 1;
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
                    TokenLoc::new(self.line, self.col, self.import_name.clone()),
                    self.acc
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
                    self.acc,
                    TokenLoc::new(self.line, self.col, self.import_name.clone())
                ));
            }
        }

        self.col += self.acc.len();
        self.acc.clear();
        Ok(())
    }

    // Pushes a new token to the output.
    fn push_tok(&mut self, tok: Token) {
        self.toks.push(tok);
        self.toks_loc
            .push(TokenLoc::new(self.line, self.col, self.import_name.clone()));
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
        let tokens = tokenize("", None, &HashMap::new(), None).unwrap().0;
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
        .0;
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
            .0;
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
        .0;
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
