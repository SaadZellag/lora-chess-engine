use std::{
    fs::File,
    io::Write,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread::{self},
    time::Instant,
};

use chess::{Board, BoardStatus, ChessMove, MoveGen, Color};
use rand::{seq::SliceRandom, thread_rng};

use crate::utils::{new_engine, GameResult};
use crate::binpack::{BinPackWriter, GameEntry};

// Chess openings last about 10 moves
// https://www.chess.com/forum/view/chess-openings/how-many-moves-does-an-opening-consist-of#:~:text=It%20is%20normally%20about%2010,become%20obsessed%20by%20opening%20theory.
// Starting after openings since engines generally have opening books
const DEPTH_START: usize = 10;

#[derive(Debug, Clone)]
pub struct GameOptions {
    pub num_positions: usize,
    pub nodes: usize,
    pub threads: usize,
    pub output_file: String,
}

pub fn play(options: GameOptions) {
    println!(
        "Generating {} positions with {} thread(s) to {}",
        options.num_positions, options.threads, options.output_file
    );

    // Open output file
    let output_file = File::create(&options.output_file)
        .expect("Failed to create output file");
    let output = Arc::new(Mutex::new(BinPackWriter::new(output_file)
        .expect("Failed to create BinPackWriter")));

    let total_count = Arc::new(AtomicUsize::new(0));
    let start = Instant::now();

    let mut threads = Vec::with_capacity(options.threads);

    for _ in 0..options.threads {
        let total_count = total_count.clone();
        let output = output.clone();
        let nodes = options.nodes;
        let num_positions = options.num_positions;

        let thread = thread::spawn(move || {
            while num_positions > total_count.load(Ordering::Relaxed) {
                let result = play_game(nodes);

                if let Ok(mut writer) = output.lock() {
                    let game_entry = GameEntry {
                        startpos: result.board,
                        startply: 0,
                        moves: result.moves,
                        result: match result.winner {
                            Some(Color::White) => crate::binpack::GameResult::WhiteWins,
                            Some(Color::Black) => crate::binpack::GameResult::BlackWins,
                            None => crate::binpack::GameResult::Draw,
                        },
                    };

                    let num_positions = game_entry.moves.len();

                    writer.write_game(&game_entry)
                        .expect("Failed to write game to output file");

                    total_count.fetch_add(num_positions, Ordering::Relaxed);
                }

                let completed = total_count.load(Ordering::Relaxed);
                let percentage = completed as f64 / num_positions as f64 * 100.0;
                let elapsed = start.elapsed();
                let eta = (elapsed * (num_positions - completed) as u32) / (completed as u32).max(1);

                let eta_mins = (eta.as_secs() / 60) % 60;
                let eta_hours = eta.as_secs() / 3600;

                print!(
                    "\r\x1b[K{:.2}% Done! {:.0} pos/sec. ETA: {}h {}min",
                    percentage,
                    completed as f64 / elapsed.as_secs_f64(),
                    eta_hours,
                    eta_mins
                );

                std::io::stdout().flush().unwrap();
            }
        });
        threads.push(thread);
    }

    for thread in threads {
        thread.join().unwrap();
    }

    println!("\nCompleted!");
}

fn play_game(nodes: usize) -> GameResult {
    let start_pos = generate_board();

    let mut board = start_pos;
    let mut moves = Vec::new();
    let mut history = Vec::new();

    let mut engine = new_engine(nodes);

    while board.status() == BoardStatus::Ongoing {
        if repetitions(&history, &board) >= 3 {
            break;
        }

        if let Some(res) = engine.best_move(board, &history) {
            moves.push((res.best_move, res.eval));
            board = board.make_move_new(res.best_move);
            history.push(board.get_hash());
        } else {
            eprintln!(
                "ERROR: Position |{}| with {} possible moves got no moves from the engine",
                board.to_string(),
                MoveGen::new_legal(&board).len()
            );
            return play_game(nodes);
        }

        if moves.len() >= 150 {
            // Cannot be bothered with implementing the full logic of checking whether it's a draw
            return play_game(nodes);
        }
    }

    let winner = match board.status() {
        BoardStatus::Ongoing => None, // Draw by repetition
        BoardStatus::Stalemate => None,
        BoardStatus::Checkmate => Some(!board.side_to_move()),
    };

    GameResult {
        board: start_pos,
        moves,
        winner,
    }
}

// Generates a board with DEPTH_START random moves played on it
fn generate_board() -> Board {
    let mut board = Board::default();

    for _ in 0..DEPTH_START {
        let moves: Vec<ChessMove> = MoveGen::new_legal(&board).collect();

        if moves.is_empty() {
            return generate_board();
        }

        let mv = *moves.choose(&mut thread_rng()).unwrap();

        board = board.make_move_new(mv);
    }

    if board.status() != BoardStatus::Ongoing {
        return generate_board();
    }

    board
}

fn repetitions(history: &[u64], board: &Board) -> usize {
    history
        .iter()
        .rev()
        .step_by(2)
        .skip(1)
        .filter(|&&hash| hash == board.get_hash())
        .count()
}
