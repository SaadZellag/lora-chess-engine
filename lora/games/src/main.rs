#![allow(stable_features)]
use clap::{Parser, Subcommand};

use crate::{
    play_games::{play, GameOptions},
    test::test,
    bench::bench,
    compute::compute,
};

mod play_games;
mod test;
mod utils;
mod bench;
mod compute;
mod binpack;

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

        /// Number of nodes to search per move (optional, defaults to 40k)
        #[arg(value_name = "NODES", default_value_t = 40_000)]
        nodes: usize,

        /// Number of threads to use for parallel game generation
        #[arg(value_name = "THREADS")]
        threads: usize,

        /// Output file path for training data (optional, defaults to training_data.binpack)
        #[arg(value_name = "OUTPUT", default_value = "training_data.binpack")]
        output_file: String,
    },

    /// Compute the scaling factor for the evaluation function based on a dataset of games
    Compute {
        #[arg(short, long)]
        binpack_files: Vec<String>,

        /// How often before the scaling factor is recomputed)
        #[arg(short, long, default_value_t = 10000000)]
        recompute_interval: usize,

        #[arg(short, long, default_value_t = false)]
        show_graph: bool,
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
            nodes,
            threads,
            output_file,
        } => {
            let options = GameOptions {
                num_positions,
                nodes,
                threads,
                output_file,
            };
            play(options);
        }
        Commands::Test { binpack_file } => {
            test(&binpack_file);
        },
        Commands::Compute { binpack_files, recompute_interval, show_graph } => {
            compute(&binpack_files, recompute_interval, show_graph);
        },
        Commands::Bench { path } => {
            bench(&path);
        }
    }
}
