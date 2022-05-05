use clap::{ArgGroup, Parser};

use std::collections::HashMap;
use std::io::Read;

mod annotater;
mod data;
mod lexer;
mod parser;
mod simplifier;

macro_rules! load_lib {
    ($a:expr) => {
        {
            let mut lib = HashMap::new();
            lib.insert($a.to_owned(), include_str!(concat!("../", $a)).to_owned());
            lib
        }
    };

    ($a:expr, $b:expr) => {
        {
            let mut lib = load_lib!($a);
            lib.insert($b.to_owned(), include_str!(concat!("../", $b)).to_owned());
            lib
        }
    };

    ($a:expr, $($b:tt)*) => {
        {
            let mut lib = load_lib!($($b)*);
            lib.insert($a.to_owned(), include_str!(concat!("../", $a)).to_owned());
            lib
        }
    }
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(group(
    ArgGroup::new("input")
        .required(true)
        .args(&["path", "stdin"]),
))]
struct Cli {
    /// The path to the file to be compiled.
    #[clap(parse(from_os_str))]
    path: Option<std::path::PathBuf>,
    /// The format used to print the resulting turing machine.
    #[clap(short, long)]
    format: Option<String>,

    /// Should the input be read from stdin instead of a file?
    #[clap(short = 'i', long)]
    stdin: bool,

    /// Should the tokens be printed?
    #[clap(short, long)]
    tokens: bool,
    /// Should the AST be printed?
    #[clap(short, long)]
    parser: bool,
    /// Should the annotated AST be printed?
    #[clap(short, long)]
    annotated: bool,
    /// Should the simplified AST be printed?
    #[clap(short, long)]
    simplified: bool,
}

fn compile(args: &Cli, lib: &HashMap<String, String>) -> Result<(), String> {
    // Tokenize input.
    let toks = if args.stdin {
        let mut src = String::new();
        std::io::stdin().read_to_string(&mut src).unwrap();
        let dir = std::env::current_dir().ok();
        lexer::tokenize(&src, dir.as_ref().map(|s| s.as_path()), lib, None)
    } else {
        lexer::tokenize_from_file(&args.path.as_ref().unwrap(), lib, None)
    }
    .map_err(|e| format!("Lexer error: {}", e))?;
    if args.tokens {
        eprintln!("----------- Tokens -----------");
        toks.iter().for_each(|(tok, _)| eprint!("{} ", tok));
        eprintln!("");
    }

    // Parse the tokens and generate the abstract syntax tree.
    let ast = parser::parse(toks).map_err(|e| format!("Parser error: {}", e))?;
    if args.parser {
        eprintln!("------------ AST -------------");
        eprintln!("{}", ast);
        eprintln!("");
    }

    // Apply intermediate simplifications on the AST:
    // - replace let statements with function applications
    // - dedup identifiers
    // - remove unused variables
    // - remove identity applications
    let ast = simplifier::let_remover::remove_lets(ast);
    let ast = simplifier::unused_remover::remove_unused(ast);
    let ast = simplifier::id_dedup::dedup_ids(ast);
    // TODO - remove identity applications
    if args.simplified {
        eprintln!("-------- Simplified AST --------");
        eprintln!("{}", ast);
        eprintln!("");
    }

    // Annotate the AST with type information.
    let ast = annotater::annotate(ast).map_err(|e| format!("Annotater error: {}", e))?;
    if args.annotated {
        eprintln!("-------- Annotated AST--------");
        eprintln!("{:#}", ast);
        eprintln!("");
    }

    // Simplify the AAST further:
    // - apply every non tape application.
    // - move matches so that they are at the root of the function expressions.
    // - remove matches which are used as arguments to other matches.
    let ast = simplifier::applier::do_applications(ast);
    let ast = simplifier::match_mover::move_matches(ast);
    //let ast = match_merger::merge_matches(ast)?;
    if args.simplified {
        eprintln!("-------- Simplified AAST --------");
        eprintln!("{:#}", ast);
        eprintln!("");
    }

    Ok(())
}

fn main() {
    let args = Cli::parse();

    // Load the standard library files.
    let lib = load_lib!("std/bool.tmc", "std/iter.tmc");

    // Compile with the input arguments and the standard library.
    std::process::exit(if let Err(err) = compile(&args, &lib) {
        eprintln!("Compilation failed: {}", err);
        1
    } else {
        eprintln!("Compilation successful!");
        0
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compiler_tests() {
        let lib = load_lib!("std/bool.tmc", "std/iter.tmc");

        // Compile every program in the tests directory.
        for entry in std::fs::read_dir("tests").unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                let path = entry.path();
                let name = path.file_name().unwrap().to_str().unwrap();

                if name.ends_with(".tmc") {
                    let args = Cli {
                        path: Some(path.clone()),
                        format: None,
                        stdin: false,
                        tokens: false,
                        parser: false,
                        annotated: false,
                        simplified: false,
                    };

                    if name.contains("fail") {
                        if compile(&args, &lib).is_ok() {
                            panic!("Test program {} should have failed!", name);
                        }
                    } else {
                        if let Err(err) = compile(&args, &lib) {
                            panic!("Test program {} should have compiled! Instead, got error: {}", name, err);
                        }
                    }
                }
            }
        }
    }
}
