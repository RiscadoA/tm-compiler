use clap::Parser;

mod data;

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
    let _args = Cli::parse();
    // TODO
}
