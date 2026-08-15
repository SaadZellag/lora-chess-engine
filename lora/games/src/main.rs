#![allow(stable_features)]
use clap::{Parser, Subcommand};

use crate::{
    parse_games::{parse, ParseOptions},
    play_games::{play, GameOptions},
    test::test,
    bench::bench,
};

mod parse_games;
mod play_games;
mod test;
mod utils;
mod bench;

#[derive(Parser)]
#[command(name = "games")]
#[command(about = "Chess training data generation tool", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Play chess games to generate training data
    Play {
        /// Number of positions to generate
        #[arg(value_name = "NUM")]
        num_positions: usize,

        /// Number of threads to use for parallel game generation
        #[arg(value_name = "THREADS")]
        threads: usize,
    },

    /// Parse FEN positions from file to generate training data
    Parse {
        /// Number of threads to use for parallel parsing
        #[arg(value_name = "THREADS")]
        threads: usize,

        /// Maximum number of positions to parse (optional, parses all if not specified)
        #[arg(value_name = "MAX")]
        max_positions: Option<usize>,
    },

    /// Test mode for validating data generation
    Test {
        #[arg(value_name = "BINPACK_FILE")]
        binpack_file: String,
    },

    /// Benchmark various things
    Bench {
        /// Path to the file to benchmark
        #[arg(value_name = "PATH")]
        path: String,
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Play {
            num_positions,
            threads,
        } => {
            let options = GameOptions {
                num_positions,
                threads,
            };
            play(options);
        }
        Commands::Parse {
            threads,
            max_positions,
        } => {
            let options = ParseOptions {
                threads,
                max_positions,
            };
            parse(options);
        }
        Commands::Test { binpack_file } => {
            test(&binpack_file);
        },
        Commands::Bench { path } => {
            bench(&path);
        }
    }
}
