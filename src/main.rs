use clap::Parser;

use std::collections::HashMap;

mod data;
mod lexer;

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
    let tokens = lexer::tokenize_from_file(&args.path, &HashMap::new(), None)
        .map_err(|e| format!("Lexer error: {}", e))
        .unwrap();
    tokens.iter().for_each(|(tok, _)| print!("{} ", tok));
    println!("");
}
