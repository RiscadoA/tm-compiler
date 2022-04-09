use clap::Parser;

use std::collections::HashMap;

mod ast;
mod data;
mod lexer;
mod parser;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The path to the file to be compiled.
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf,
    /// The format used to print the resulting turing machine.
    #[clap(short, long)]
    format: Option<String>,
}

fn main() {
    let args = Cli::parse();
    // Tokenize input.
    let toks = lexer::tokenize_from_file(&args.path, &HashMap::new(), None)
        .map_err(|e| format!("Lexer error: {}", e))
        .unwrap();
    println!("----------- Tokens -----------");
    toks.iter().for_each(|(tok, _)| print!("{} ", tok));
    println!("");

    // Parse the tokens and generate the abstract syntax tree.
    println!("------------ AST -------------");
    let ast = parser::parse(toks)
        .map_err(|e| format!("Parser error: {}", e))
        .unwrap();
    println!("{}", ast);

    // Annotate the AST with types.
    println!("------------ TAST ------------");
    let tast = ast::type_check(ast)
        .map_err(|e| format!("Type checker error: {}", e))
        .unwrap();
}
