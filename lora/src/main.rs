mod bench;

use cozy_chess::{Board, Move};
use cozy_uci::{UciFormatOptions, command::{UciCommand, UciGoParams}, remark::{UciRemark, UciIdInfo, UciInfo, UciScore, UciScoreKind}};
use engine::{LoraEngine, SearchOptions, SearchPosition, SearchResult, Eval, TranspositionTable};
use std::{
    io::{BufRead, Write, stdin},
    sync::{Arc, atomic::AtomicBool},
    time::{Instant, Duration},
};


fn print_remark(remark: UciRemark, options: &UciFormatOptions) {
    println!("{}", remark.format(options));
    std::io::stdout().flush().unwrap();
}

fn calculate_time_to_think(position: &SearchPosition, go_params: &UciGoParams) -> Duration {
    // If there's a specific move time, use that
    if let Some(movetime) = go_params.movetime {
        return movetime;
    }

    // If infinite, use a very large time
    if go_params.infinite {
        return Duration::from_secs(31_557_600); // ~1 year
    }

    // Determine whose turn it is based on number of moves played
    // Even number of moves => white's turn, odd => black's turn
    let is_white_to_move = position.moves_played.len() % 2 == 0;
    
    let our_time = if is_white_to_move {
        go_params.wtime
    } else {
        go_params.btime
    };
    let our_increment = if is_white_to_move {
        go_params.winc
    } else {
        go_params.binc
    };

    let mut time_to_use = Duration::ZERO;

    // 5% of remaining time
    if let Some(time) = our_time {
        time_to_use = time_to_use + Duration::from_millis(time.as_millis() as u64 / 20);
    }

    // 50% of increment
    if let Some(inc) = our_increment {
        time_to_use = time_to_use + Duration::from_millis(inc.as_millis() as u64 / 2);
    }

    // Default fallback
    if time_to_use == Duration::ZERO {
        time_to_use = Duration::from_secs(5);
    }

    time_to_use
}

fn print_options(options: &UciFormatOptions) {
    use cozy_uci::remark::UciOptionInfo;
    
    print_remark(UciRemark::Option {
        name: "Hash".to_string(),
        info: UciOptionInfo::Spin {
            default: 1,
            min: 1,
            max: 1024,
        },
    }, options);
    
    print_remark(UciRemark::Option {
        name: "Threads".to_string(),
        info: UciOptionInfo::Spin {
            default: 1,
            min: 1,
            max: 1,
        },
    }, options);
}

#[derive(Clone)]
struct UCIHandler {
    start: Instant,
    time_allowed: Duration,
    abort_flag: Arc<AtomicBool>,
    last_result: Option<SearchResult>,
    uci_options: UciFormatOptions,
}

impl engine::SearchHandler for UCIHandler {
    fn new_result(&mut self, result: engine::SearchResult) {
        let elapsed = self.start.elapsed();
        let nodes = result.stats.nodes_visited;
        let nps = (nodes as f64 / elapsed.as_secs_f64()) as u64;

        let score = match result.eval {
            Eval::MateIn(x) => UciScore {
                cp: None,
                mate: Some(x as i32),
                wdl: None,
                kind: UciScoreKind::Exact,
            },
            Eval::MatedIn(x) => UciScore {
                cp: None,
                mate: Some(-(x as i32)),
                wdl: None,
                kind: UciScoreKind::Exact,
            },
            Eval::CentiPawn(x) => UciScore {
                cp: Some(x as i32),
                mate: None,
                wdl: None,
                kind: UciScoreKind::Exact,
            },
        };

        let pv: Vec<Move> = result
            .pv
            .iter()
            .cloned()
            .collect();

        let info = UciInfo {
            depth: Some(result.stats.depth as u32),
            seldepth: Some(result.stats.sel_depth as u32),
            time: Some(elapsed),
            nodes: Some(nodes),
            pv: if pv.is_empty() { None } else { Some(pv) },
            multipv: None,
            score: Some(score),
            currmove: Some(result.best_move),
            currmovenumber: None,
            hashfull: Some(result.hashfull as u16),
            nps: Some(nps),
            tbhits: Some(result.stats.tbl_hits),
            sbhits: None,
            cpuload: None,
            string: None,
            refutation: None,
            currline: None,
        };

        print_remark(UciRemark::Info(info), &self.uci_options);
        self.last_result = Some(result);
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

    let options = UciFormatOptions::default();

    'outer: for line in lines {
        // Parse the UCI command
        match UciCommand::parse_from(&line, &options) {
            Ok(cmd) => {
                match cmd {
                    UciCommand::Uci => {
                        print_remark(UciRemark::Id(UciIdInfo::Name(format!(
                            "Lora {} hash {}",
                            env!("CARGO_PKG_VERSION"),
                            env!("GIT_HASH")
                        ))), &options);
                        print_remark(UciRemark::Id(UciIdInfo::Author("Saad2442".to_string())), &options);
                        print_options(&options);
                        print_remark(UciRemark::UciOk, &options);
                    }

                    UciCommand::Debug(_) => {
                        // Debug not implemented
                    }

                    UciCommand::IsReady => {
                        print_remark(UciRemark::ReadyOk, &options);
                    }

                    UciCommand::Position { init_pos, moves } => {
                        position.board = init_pos.into();
                        position.moves_played = moves;
                    }

                    UciCommand::SetOption { name, value } => match name.to_lowercase().as_ref() {
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

                    UciCommand::UciNewGame => {
                        position.board = Board::default();
                        position.moves_played.clear();
                        abort_flag.store(false, std::sync::atomic::Ordering::Relaxed);
                        transposition_table = TranspositionTable::new(engine.options.tt_size_bytes);
                    }

                    UciCommand::Stop => {
                        abort_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                    }

                    UciCommand::PonderHit => {
                        // PonderHit not supported
                    }

                    UciCommand::Go(go_params) => {
                        let time_allowed = calculate_time_to_think(&position, &go_params);
                        
                        let mut search_options = SearchOptions::new();
                        
                        if let Some(depth) = go_params.depth {
                            search_options.max_depth = depth.min(255) as u8;
                        }
                        if let Some(nodes) = go_params.nodes {
                            search_options.max_nodes = nodes;
                        }
                        if let Some(mate) = go_params.mate {
                            search_options.mate_search_depth = Some(mate.min(255) as u8);
                        }
                        if let Some(moves) = go_params.searchmoves {
                            if !moves.is_empty() {
                                search_options.moves_to_search = Some(moves);
                            }
                        }

                        let mut handler = UCIHandler {
                            start: Instant::now(),
                            time_allowed,
                            abort_flag: abort_flag.clone(),
                            last_result: None,
                            uci_options: options.clone(),
                        };

                        if let Some(result) = engine.search(position.clone(), search_options, &mut handler, &mut transposition_table) {
                            print_remark(UciRemark::BestMove {
                                mv: result.best_move,
                                ponder: None,
                            }, &options);
                        }
                    }

                    UciCommand::Quit => {
                        break 'outer;
                    }
                }
            }
            Err(_) => {
                // Silently ignore parse errors for now
            }
        }
    }
}
