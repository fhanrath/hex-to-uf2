use std::path::PathBuf;

use clap::Parser;
use hex_to_uf2::{families::ChipFamily, hex_to_uf2_file};

/// Converts hex files to uf2
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    hex_file: PathBuf,
    output_path: PathBuf,
    family: Option<ChipFamily>,
}

fn main() {
    let args = Cli::parse();

    hex_to_uf2_file(&args.hex_file, &args.output_path, args.family).unwrap();
}
