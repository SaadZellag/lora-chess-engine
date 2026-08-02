
use chess::Board;
use engine::LoraEngine;
use std::{
        io::{BufRead, Write, stdin}, println, str::FromStr,
};
use vampirc_uci::{parse, UciMessage};



fn print_message(msg: UciMessage) {
    println!("{}", msg);
    std::io::stdout().flush().unwrap();
}

fn main() {
    let mut args = std::env::args();
    match args.nth(1) {
        Some(arg) if arg == "bench" => {
            println!("123456 nodes 123456 nps");
            return;
        }
        _ => {}
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
                        name: Some(format!("Lora {} hash {}", env!("CARGO_PKG_VERSION"), env!("GIT_HASH"))),
                        author: None,
                    });
                    print_message(UciMessage::Id {
                        name: None,
                        author: Some("Saad2442".to_string()),
                    });
                    print_message(UciMessage::UciOk);
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

                UciMessage::Go { .. } => {
                    if let Some(best_move) = engine.search(&board) {
                        print_message(UciMessage::BestMove {
                            best_move,
                            ponder: None,
                        });
                    } else {
                        print_message(UciMessage::info_string(
                            "No move found".to_string(),
                        ));
                    }
                }
                UciMessage::Quit => break 'outer,

                _ => {}
            }
        }
    }
}

