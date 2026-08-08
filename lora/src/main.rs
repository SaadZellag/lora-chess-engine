mod bench;

use chess::Board;
use engine::{LoraEngine, SearchOptions, SearchPosition};
use std::{
    io::{BufRead, Write, stdin}, println, rc::Rc, str::FromStr,
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

struct Handler {}

impl engine::SearchHandler for Handler {
    fn new_result(&mut self, result: engine::SearchResult) {
        // Handle new search result
    }

    fn should_stop(&self) -> bool {
        false
    }
}

fn main() {
    let mut args = std::env::args();
    if let Some(arg) = args.nth(1) {
        if arg.as_str() == "bench" {
            return bench::bench();
        }
    }

    let mut position = SearchPosition::new();
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
                        position.board = Board::default();
                    } else if let Some(fen_str) = fen {
                        if let Ok(parsed) = Board::from_str(&fen_str.0) {
                            position.board = parsed;
                        }
                    }

                    position.moves_played = moves;

                }

                UciMessage::SetOption { name, value: _ } => match name.to_lowercase().as_ref() {
                    "hash" => {

                    }
                    "threads" => {
                        // TODO
                    }
                    _ => {}
                },

                UciMessage::UciNewGame => {
                    position.board = Board::default();
                }

                UciMessage::Stop => {
                    // TODO
                }

                UciMessage::PonderHit => {
                    print_message(UciMessage::info_string(
                        "PonderHit not supported".to_string(),
                    ));
                }

                UciMessage::Go { search_control, ..  } => {
                    let mut search_options = SearchOptions::new();
                    if let Some(control) = search_control {
                        search_options.max_depth = control.depth.unwrap_or(search_options.max_depth);
                        search_options.max_nodes = control.nodes.unwrap_or(search_options.max_nodes);
                        search_options.mate_search_depth = control.mate;
                        search_options.moves_to_search = if control.search_moves.is_empty() {
                            None
                        } else {
                            Some(control.search_moves)
                        };
                    }
                    

                    if let Some(result) = engine.search(position.clone(), search_options, &mut Handler {}) {
                        print_message(UciMessage::BestMove {
                            best_move: result.best_move,
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
