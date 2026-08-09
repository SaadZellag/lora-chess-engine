mod bench;

use chess::{Board, ChessMove};
use chrono::TimeDelta;
use engine::{LoraEngine, SearchOptions, SearchPosition, SearchResult, Eval, TranspositionTable};
use std::{
    io::{BufRead, Write, stdin}, println, rc::Rc, str::FromStr,
    sync::{Arc, atomic::AtomicBool},
    time::{Instant, Duration},
};

use vampirc_uci::{UciInfoAttribute, UciMessage, UciOptionConfig, UciTimeControl, parse};

fn print_message(msg: UciMessage) {
    println!("{}", msg);
    std::io::stdout().flush().unwrap();
}

fn calculate_time_to_think(position: &SearchPosition, time_control: &Option<UciTimeControl>) -> Duration {
    match time_control {
        Some(UciTimeControl::MoveTime(move_time)) => {
            move_time.to_std().unwrap_or(Duration::from_secs(5))
        }
        Some(UciTimeControl::TimeLeft {
            white_time,
            black_time,
            white_increment,
            black_increment,
            ..
        }) => {
            // Apply moves to get current board state
            let mut current_board = position.board;
            for mv in &position.moves_played {
                current_board = current_board.make_move_new(*mv);
            }
            
            let side = current_board.side_to_move();
            let our_time = if side == chess::Color::White {
                white_time
            } else {
                black_time
            };
            let our_increment = if side == chess::Color::White {
                white_increment
            } else {
                black_increment
            };

            let mut time_to_use = Duration::ZERO;

            // 5% of remaining time
            if let Some(time) = our_time {
                let remaining_ms = time.num_milliseconds().max(0) as u64;
                time_to_use = time_to_use + Duration::from_millis(remaining_ms / 20);
            }

            // 50% of increment
            if let Some(inc) = our_increment {
                let inc_ms = inc.num_milliseconds().max(0) as u64;
                time_to_use = time_to_use + Duration::from_millis(inc_ms / 2);
            }

            time_to_use
        }
        Some(UciTimeControl::Infinite) | Some(UciTimeControl::Ponder) => {
            Duration::from_secs(31_557_600) // ~1 year
        }
        None => Duration::from_secs(5), // Default fallback
    }
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

#[derive(Clone)]
struct UCIHandler {
    start: Instant,
    time_allowed: Duration,
    abort_flag: Arc<AtomicBool>,
    last_result: Option<SearchResult>,
}

impl engine::SearchHandler for UCIHandler {
    fn new_result(&mut self, result: engine::SearchResult) {
        let depth = UciInfoAttribute::Depth(result.stats.depth);
        let sel_depth = UciInfoAttribute::SelDepth(result.stats.sel_depth);
        let std_duration = self.start.elapsed();
        let time_delta = TimeDelta::from_std(std_duration).expect("duration out of bounds");

        let time = UciInfoAttribute::Time(time_delta);
        let nodes = UciInfoAttribute::Nodes(result.stats.nodes_visited);
        let score = match result.eval {
            Eval::MateIn(x) => UciInfoAttribute::from_mate(x as i8),
            Eval::MatedIn(x) => UciInfoAttribute::from_mate(-(x as i8)),
            Eval::CentiPawn(x) => UciInfoAttribute::from_centipawns(x as i32),
        };
        let curr_move = UciInfoAttribute::CurrMove(result.best_move);
        let nps = {
            let delta = self.start.elapsed();
            let nodes = result.stats.nodes_visited;
            let nps = (nodes as f64 / delta.as_secs_f64()) as u64;
            UciInfoAttribute::Nps(nps)
        };
        let tbl_hits = UciInfoAttribute::TbHits(result.stats.tbl_hits);
        let hashfull = UciInfoAttribute::HashFull(result.hashfull as u16);
        let pv = UciInfoAttribute::Pv(
            result
                .pv
                .iter()
                .cloned()
                .take_while(|&e| e != ChessMove::default())
                .collect(),
        );

        let attributes = vec![
            depth, sel_depth, time, nodes, score, curr_move, nps, tbl_hits, hashfull, pv,
        ];
        print_message(UciMessage::Info(attributes));
    }

    fn should_stop(&self) -> bool {
        (self.abort_flag.load(std::sync::atomic::Ordering::Relaxed)
            || self.start.elapsed() >= self.time_allowed)
            && self.last_result.is_some()
    }
}

fn main() {
    let mut args = std::env::args();
    if let Some(arg) = args.nth(1) {
        if arg.as_str() == "bench" {
            return bench::bench();
        }
    }

    let abort_flag = Arc::new(AtomicBool::new(false));

    let mut position = SearchPosition::new();
    let mut engine = LoraEngine::new();
    let mut transposition_table = TranspositionTable::new(engine.options.tt_size_bytes);

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

                UciMessage::SetOption { name, value } => match name.to_lowercase().as_ref() {
                    "hash" => {
                        if let Some(val) = value {
                            if let Ok(size_mb) = val.parse::<usize>() {
                                engine.set_hash_size(size_mb);
                                transposition_table = TranspositionTable::new(engine.options.tt_size_bytes);
                            }
                        }
                    }
                    "threads" => {
                        // Ignore - single threaded only
                    }
                    _ => {}
                },

                UciMessage::UciNewGame => {
                    position.board = Board::default();
                    position.moves_played.clear();
                    abort_flag.store(false, std::sync::atomic::Ordering::Relaxed);
                    transposition_table = TranspositionTable::new(engine.options.tt_size_bytes);
                }

                UciMessage::Stop => {
                    abort_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                }

                UciMessage::PonderHit => {
                    print_message(UciMessage::info_string(
                        "PonderHit not supported".to_string(),
                    ));
                }

                UciMessage::Go { search_control, time_control  } => {
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

                    let time_allowed = calculate_time_to_think(&position, &time_control);
                    
                    let mut handler = UCIHandler {
                        start: Instant::now(),
                        time_allowed,
                        abort_flag: abort_flag.clone(),
                        last_result: None,
                    };

                    if let Some(result) = engine.search(position.clone(), search_options, &mut handler, &mut transposition_table) {
                        print_message(UciMessage::BestMove {
                            best_move: result.best_move,
                            ponder: None,
                        });
                    } else {
                        print_message(UciMessage::info_string("No move found".to_string()));
                    }
                },
                UciMessage::Quit => {
                    break 'outer;
                }
                _ => {}
            }
        }
    }
}
