use clap::{ArgGroup, Parser};

use std::collections::{HashMap, HashSet};
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

    /// The working alphabet of the turing machine.
    #[clap(short, long, required = true, multiple_values = true)]
    alphabet: Vec<String>,

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
    #[clap(short = 'A', long)]
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

    // Get the set of symbols used by the AST (including those only used during compilation)
    let alphabet = HashSet::from_iter(
        args.alphabet
            .iter()
            .map(|s| s.to_owned())
            .chain(std::iter::once("".to_owned())),
    );
    let mut const_alphabet = alphabet.clone();
    ast.collect_symbols(&mut const_alphabet);

    // Apply initial simplifications on the AST and dedup ids:
    // - replace let statements with function applications.
    // - remove match 'any' patterns.
    // - remove trivial applications.
    let ast = ast.transform(&|e| {
        let e = simplifier::let_remover::remove_lets(e);
        let e = simplifier::any_remover::remove_any(e, &const_alphabet);
        let e = simplifier::trivial_remover::remove_trivial(e);
        e
    });
    let ast = simplifier::id_dedup::dedup_ids(ast);
    if args.simplified {
        eprintln!("-------- Simplified AST --------");
        eprintln!("{}", ast);
        eprintln!("");
    }

    // Annotate the AST with the type of each expression, and check if match patterns are constant.
    let ast = annotater::type_checker::type_check(ast)
        .map_err(|e| format!("Type checker error: {}", e))?;
    annotater::const_checker::const_check(&ast)
        .map_err(|e| format!("Const checker error: {}", e))?;

    // Remove all non tape -> tape applications which can be removed before checking ownership rules.
    fn ownership_transform(e: data::Exp<annotater::Annot>) -> data::Exp<annotater::Annot> {
        let e = simplifier::applier::apply(e, |e| e.transform(&ownership_transform));
        let e = simplifier::trivial_remover::remove_trivial(e);
        e
    }
    let ast = ast.transform(&ownership_transform);

    // Check for ownership errors and resolve the types of unions.
    annotater::ownership_checker::ownership_check(&ast)
        .map_err(|e| format!("Ownership checker error: {}", e))?;
    let ast = annotater::union_resolver::resolve_unions(ast);
    if args.annotated {
        eprintln!("-------- Annotated AST --------");
        eprintln!("{:#}", ast);
        eprintln!("");
    }

    // Simplify the AAST even further:
    // - remove match captured variables.
    // - remove non tape -> tape applications
    // - remove trivial applications
    // - move matches so that they are at the root of the function expressions.
    // - match matches with symbols an input.
    // - remove duplicate match patterns.
    // - merge matches which are used as arguments to other matches.
    // - remove get applications, replacing them with match expressions.
    // - simplify get & set expressions when symbols are already known.
    // - merge match arms which have equivalent expressions.
    fn final_transform(
        e: data::Exp<annotater::Annot>,
        alphabet: &HashSet<String>,
    ) -> data::Exp<annotater::Annot> {
        let rec = |e: data::Exp<annotater::Annot>| e.transform(&|e| final_transform(e, alphabet));
        let e = simplifier::capture_remover::remove_captures(e, rec);
        let e = simplifier::applier::apply(e, rec);
        let e = simplifier::trivial_remover::remove_trivial(e);
        let e = simplifier::match_mover::move_matches(e);
        let e = simplifier::matcher::match_const(e);
        let e = simplifier::pat_dedup::dedup_patterns(e);
        let e = simplifier::match_merger::merge_matches(e);
        let e = simplifier::get_remover::remove_gets(e, alphabet);
        let e = simplifier::match_deduper::dedup_matches(e, rec);
        let e = simplifier::arm_merger::merge_arms(e);
        e
    }
    let ast = ast.transform(&|e| final_transform(e, &alphabet));
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
    let lib = load_lib!("std/bool.tmc", "std/iter.tmc", "std/check.tmc");

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
        let lib = load_lib!("std/bool.tmc", "std/iter.tmc", "std/check.tmc");

        // Compile every program in the tests directory.
        for entry in std::fs::read_dir("tests").unwrap() {
            let entry = entry.unwrap();
            if entry.file_type().unwrap().is_file() {
                let path = entry.path();
                let name = path.file_name().unwrap().to_str().unwrap();

                if name.ends_with(".tmc") {
                    let args = Cli {
                        alphabet: ["0", "1", "i", "z", "a", "b"]
                            .iter()
                            .map(|s| s.to_string())
                            .collect(),
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
                            panic!(
                                "Test program {} should have compiled! Instead, got error: {}",
                                name, err
                            );
                        }
                    }
                }
            }
        }
    }
}
