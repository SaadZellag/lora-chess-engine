#![feature(slice_as_chunks)]
use std::env::{args, Args};

use crate::{
    parse_games::{parse, ParseOptions},
    play_games::{play, GameOptions},
    test::test,
};

mod parse_games;
mod play_games;
mod test;
mod utils;

fn main() {
    let mut args = args();
    args.next(); // Skipping program name

    match args.next().as_deref() {
        Some("play") => run_games(args),
        Some("parse") => run_parse(args),
        Some("test") => test(),
        Some(opt) => eprintln!("Invalid option {}", opt),
        None => eprintln!("Put an option with args"),
    }
}

fn run_games(mut args: Args) {
    let num_positions: usize = args.next().unwrap().parse().unwrap();
    let threads: usize = args.next().unwrap().parse().unwrap();

    let options = GameOptions {
        num_positions,
        threads,
    };

    play(options);
}

fn run_parse(mut args: Args) {
    let threads: usize = args.next().unwrap().parse().unwrap();
    let games: Option<usize> = args.next().map(|s| s.parse().unwrap());

    let options = ParseOptions {
        threads,
        max_positions: games,
    };

    parse(options);
}
