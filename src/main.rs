use clap::Parser;

use std::collections::HashMap;

mod annotater;
mod data;
mod lexer;
mod parser;
mod simplifier;

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
    println!("{:#}", ast);

    // Check if the AST is valid and annotate it with extra information.
    println!("-------- Annotated AST--------");
    let ast = annotater::annotate(ast)
        .map_err(|e| format!("Annotater error: {}", e))
        .unwrap();
    println!("{:#}", ast);

    // Simplify the abstract syntax tree.
    println!("------- Simplified AST -------");
    let ast = simplifier::simplify(ast);
    println!("{:#}", ast);
}
