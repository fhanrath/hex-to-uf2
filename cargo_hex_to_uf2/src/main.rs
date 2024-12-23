use std::path::PathBuf;

use clap::{Parser, Subcommand};
use hex_to_uf2::{families::ChipFamily, hex_to_uf2_file};

/// Converts hex files to uf2
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Adds files to myapp
    #[command(name = "hex-to-uf2")]
    HexToUf2 {
        /// Path to input hex file
        #[arg(short, long)]
        input_path: PathBuf,
        /// Path to output uf2 file
        #[arg(short, long)]
        output_path: PathBuf,
        /// family of chips
        #[arg(short, long)]
        family: Option<ChipFamily>,
        /// Log debug level
        #[arg(short, long, default_value_t = false)]
        debug: bool,
    },
}

fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::HexToUf2 {
            input_path,
            output_path,
            family,
            debug,
        } => {
            let log_level = match debug {
                true => tracing::Level::DEBUG,
                false => tracing::Level::INFO,
            };

            tracing_subscriber::fmt().with_max_level(log_level).init();

            tracing::debug!("{:?}", &args);

            hex_to_uf2_file(input_path, output_path, family.clone()).unwrap();
        }
    }
}
