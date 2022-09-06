use crate::engine::Engine;
use clap::Parser;
use std::path::{Path, PathBuf};

mod engine;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    /// The input file
    #[clap(parse(from_os_str))]
    input_file: PathBuf,
}

fn main() {
    let args = Cli::parse();
    let mut engine = Engine::from_file(Path::new(args.input_file.as_path()));
    engine.process();
    engine.output_clients()
}
