use std::{
    collections::HashSet,
    fs::OpenOptions,
    io::{BufWriter, Write},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread::{self},
    time::Instant,
};

use chess::{Board, BoardStatus, ChessMove, MoveGen};

use games::create_io;
use rand::{seq::SliceRandom, thread_rng, Rng};

use crate::utils::{new_engine, shared, GameResult, ENTRY_SIZE_BYTES};

// Chess openings last about 10 moves
// https://www.chess.com/forum/view/chess-openings/how-many-moves-does-an-opening-consist-of#:~:text=It%20is%20normally%20about%2010,become%20obsessed%20by%20opening%20theory.
// Starting after openings since engines generally have opening books
const DEPTH_START: usize = 10;

#[derive(Debug, Clone, Copy)]
pub struct GameOptions {
    pub num_positions: usize,
    pub threads: usize,
}

pub fn play(options: GameOptions) {
    let train_path = "training_data.bin";
    let val_path = "val_training_data.bin";

    println!(
        "Playing ~{} games with {} thread(s)",
        options.num_positions / 100,
        options.threads
    );

    let train_output =
        shared(create_io!(train_path, BufWriter, create true, write true, truncate true));
    let val_output =
        shared(create_io!(val_path, BufWriter, create true, write true, truncate true));

    let seen = Arc::new(Mutex::new(HashSet::with_capacity(
        options.num_positions / 10,
    )));
    let total_count = Arc::new(AtomicUsize::new(1));
    let start = Instant::now();

    let mut threads = Vec::with_capacity(options.threads);

    for _ in 0..options.threads {
        let total_count = total_count.clone();
        let train_output = train_output.clone();
        let val_output = val_output.clone();
        let seen = seen.clone();

        let thread = thread::spawn(move || {
            while options.num_positions > total_count.load(Ordering::Relaxed) {
                let result = play_game();

                let bin_data = result.to_bin(seen.clone());

                // Splitting 90% train 10% test
                let output = match thread_rng().gen_bool(0.9) {
                    true => &train_output,
                    false => &val_output,
                };

                if let Ok(mut out) = output.lock() {
                    out.write_all(&bin_data).unwrap();
                    total_count.fetch_add(bin_data.len() / ENTRY_SIZE_BYTES, Ordering::Relaxed);

                    // out.flush().unwrap();
                }

                let completed = total_count.load(Ordering::Relaxed);
                let percentage = completed as f64 / options.num_positions as f64 * 100.0;
                let elapsed = start.elapsed();
                let eta = (elapsed * (options.num_positions - completed) as u32) / completed as u32;

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
}

fn play_game() -> GameResult {
    let start_pos = generate_board();

    let mut board = start_pos;
    let mut moves = Vec::new();
    let mut history = Vec::new();

    let mut engine = new_engine(board, history.clone());

    while board.status() == BoardStatus::Ongoing {
        if repetitions(&history, &board) >= 3 {
            break;
        }

        if let Some(res) = engine.best_move() {
            moves.push((res.best_move, res.eval));
            board = board.make_move_new(res.best_move);
            history.push(board.get_hash());
        } else {
            eprintln!(
                "ERROR: Position |{}| with {} possible moves got no moves from the engine",
                board.to_string(),
                MoveGen::new_legal(&board).len()
            );
            return play_game();
        }

        engine.set_position(board, &history);
        engine.set_handler(Default::default());

        if moves.len() >= 150 {
            // Cannot be bothered with implementing the full logic of checking whether it's a draw
            return play_game();
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
