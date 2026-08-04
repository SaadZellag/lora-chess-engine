mod bench;

use chess::Board;
use engine::LoraEngine;
use std::{
    io::{stdin, BufRead, Write},
    println,
    str::FromStr,
};

use vampirc_uci::{parse, UciMessage, UciOptionConfig};

fn print_message(msg: UciMessage) {
    println!("{}", msg);
    std::io::stdout().flush().unwrap();
}

fn print_options() {
    print_message(UciMessage::Option(UciOptionConfig::Spin {
        name: "Hash".to_string(),
        default: Some(1),
        min: Some(1),
        max: Some(1024),
    }));
    print_message(UciMessage::Option(UciOptionConfig::Spin {
        name: "Threads".to_string(),
        default: Some(1),
        min: Some(1),
        max: Some(1),
    }));
}

fn main() {
    let mut args = std::env::args();
    if let Some(arg) = args.nth(1) {
        match arg.as_str() {
            "bench" => return bench::bench(),
            _ => {}
        }
    }

    let mut board = Board::default();
    let mut engine = LoraEngine::new();

    let stdin = stdin();
    let lines = stdin.lock().lines().map(|l| l.unwrap_or_default());

    'outer: for line in lines {
        for msg in parse(&line) {
            match msg {
                UciMessage::Uci => {
                    print_message(UciMessage::Id {
                        name: Some(format!(
                            "Lora {} hash {}",
                            env!("CARGO_PKG_VERSION"),
                            env!("GIT_HASH")
                        )),
                        author: None,
                    });
                    print_message(UciMessage::Id {
                        name: None,
                        author: Some("Saad2442".to_string()),
                    });
                    print_options();
                    print_message(UciMessage::UciOk);
                }

                UciMessage::Debug(_) => {
                    print_message(UciMessage::info_string("Debug not supported".to_string()));
                }

                UciMessage::IsReady => print_message(UciMessage::ReadyOk),

                UciMessage::Position {
                    startpos,
                    fen,
                    moves,
                } => {
                    if startpos {
                        board = Board::default();
                    } else if let Some(fen_str) = fen {
                        if let Ok(parsed) = Board::from_str(&fen_str.0) {
                            board = parsed;
                        }
                    }

                    for mv in moves {
                        board = board.make_move_new(mv);
                    }
                }

                UciMessage::SetOption { name, value } => match name.to_lowercase().as_ref() {
                    "hash" => {
                        // TODO
                    }
                    "threads" => {
                        // TODO
                    }
                    _ => {}
                },

                UciMessage::UciNewGame => {
                    board = Board::default();
                }

                UciMessage::Stop => {
                    // TODO
                }

                UciMessage::PonderHit => {
                    print_message(UciMessage::info_string(
                        "PonderHit not supported".to_string(),
                    ));
                }

                UciMessage::Go { .. } => {
                    if let Some(best_move) = engine.search(&board) {
                        print_message(UciMessage::BestMove {
                            best_move,
                            ponder: None,
                        });
                    } else {
                        print_message(UciMessage::info_string("No move found".to_string()));
                    }
                }
                _ => {}
            }
        }
    }
}
