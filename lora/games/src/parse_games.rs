use std::{
    collections::HashSet,
    fs::{self, File, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Read, Write},
    path::Path,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread::{self},
    time::Instant,
};

use chess::Board;
use engine::Eval;
use games::{
    clone, create_io,
    utils::{new_engine, ENTRY_SIZE_BYTES},
};
use rand::{thread_rng, Rng};

use crate::utils::shared;
use crate::utils::GameResult;

#[derive(Debug, Clone, Copy)]
pub struct ParseOptions {
    pub threads: usize,
    pub max_positions: Option<usize>,
}

pub fn parse(options: ParseOptions) {
    let file_path = "training_input.fens";
    let train_path = "training_data.bin";
    let val_path = "val_training_data.bin";

    let mut threads = Vec::new();

    let total_positions = if let Some(games) = options.max_positions {
        games
    } else {
        BufReader::new(File::open(file_path).unwrap())
            .lines()
            .count()
    };

    println!(
        "Parsing {} games with {} thread(s)",
        total_positions, options.threads
    );

    let input = shared(create_io!(file_path, BufReader, read true).lines());

    let train_output =
        shared(create_io!(train_path, BufWriter, create true, write true, truncate true));
    let val_output =
        shared(create_io!(val_path, BufWriter, create true, write true, truncate true));

    let total_position_count = Arc::new(AtomicUsize::new(0));
    let seen = Arc::new(Mutex::new(HashSet::with_capacity(total_positions / 10)));
    let start = Instant::now();

    for _ in 0..options.threads {
        clone!(total_position_count input train_output val_output seen);

        threads.push(thread::spawn(move || 'outer: loop {
            let mut lines = vec![];
            if let Ok(mut input) = input.lock() {
                for _ in 0..1000 {
                    if let Some(Ok(line)) = input.next() {
                        lines.push(line);
                    } else {
                        break 'outer;
                    }
                }
            } else {
                break;
            };

            // For manual abborts
            if total_position_count.load(Ordering::Relaxed) > total_positions {
                break;
            }

            // let parsed: Vec<_> = lines
            //     .iter()
            //     .map(|l| parse_line(l))
            //     .flatten()
            //     .map(|p| p.to_bin(seen.clone()))
            //     .flatten()
            //     .collect();

            let parsed: Vec<u8> = parse_lines(lines)
                .iter()
                .map(|p| p.to_bin(seen.clone()))
                .flatten()
                .collect();

            // Splitting 90% train 10% test
            let output = match thread_rng().gen_bool(0.9) {
                true => &train_output,
                false => &val_output,
            };

            if let Ok(mut out) = output.lock() {
                out.write_all(&parsed).unwrap();
                total_position_count.fetch_add(parsed.len() / ENTRY_SIZE_BYTES, Ordering::Relaxed);

                out.flush().unwrap();
            }

            let completed = 1 + total_position_count.load(Ordering::Relaxed);

            let percentage = completed as f64 / total_positions as f64 * 100.0;
            let elapsed = start.elapsed();
            let eta = (elapsed * (total_positions - completed) as u32) / completed as u32;

            let eta_mins = (eta.as_secs() / 60) % 60;
            let eta_hours = eta.as_secs() / 3600;

            print!(
                "\r\x1b[K{:.2}% Done! ({} positions). ({:.1} pos/sec) ETA: {}h {}min",
                percentage,
                total_position_count.load(Ordering::Relaxed),
                completed as f64 / elapsed.as_secs_f64(),
                eta_hours,
                eta_mins
            );

            std::io::stdout().flush().unwrap();
        }));
    }

    println!();

    for thread in threads {
        thread.join().unwrap();
    }
}

fn parse_lines(lines: Vec<String>) -> Vec<GameResult> {
    let mut engine = new_engine(Board::default(), vec![]);

    lines
        .iter()
        .map(|line| {
            let mut content = line.split('|');
            let board: Board = content.next()?.parse().ok()?;

            content.next()?;
            content.next()?;

            let winner = match content.next()? {
                "1" => Some(board.side_to_move()),
                "-1" => Some(!board.side_to_move()),
                "0" => None,
                _ => return None,
            };

            engine.set_position(board, &vec![]);
            engine.set_handler(Default::default());

            let result = engine.best_move()?;

            Some(GameResult {
                board,
                moves: vec![(result.best_move, result.eval)],
                winner,
            })
        })
        .flatten()
        .collect()
}

fn parse_line(line: &str) -> Option<GameResult> {
    let mut content = line.split('|');

    let board: Board = content.next()?.parse().ok()?;

    let mv = content.next()?.parse().ok()?;
    let eval = Eval::CentiPawn(content.next()?.parse().ok()?);

    let winner = match content.next()? {
        "1" => Some(board.side_to_move()),
        "-1" => Some(!board.side_to_move()),
        "0" => None,
        _ => return None,
    };

    Some(GameResult {
        board,
        moves: vec![(mv, eval)],
        winner,
    })
}

// fn split_data_evenly<P>(file_path: P)
// where
//     P: AsRef<Path> + Copy,
// {
//     // Train to validation
//     const RATIO_NUM: i32 = 9;
//     const RATIO_DEN: i32 = 10;
//     const TRAIN_RANGE: i32 = RATIO_NUM * 2;
//     const VAL_RANGE: i32 = RATIO_DEN * 2;

//     let mut input = create_io!(file_path, BufReader, read true);
//     let mut train_output =
//         create_io!("training_data.bin", BufWriter, create true, write true, truncate true);
//     let mut val_output =
//         create_io!("val_training_data.bin", BufWriter, create true, write true, truncate true);

//     let num_entries = fs::metadata(file_path).unwrap().len() as usize / ENTRY_SIZE_BYTES;

//     let mut train_count = 0;
//     let mut total_count = 0;

//     let mut white_buff = Vec::new();
//     let mut black_buff = Vec::new();

//     let mut was_white_last = false;

//     let mut buff = [0; ENTRY_SIZE_BYTES];

//     let mut write_to_file = |buff: &[u8], total_count: i32| {
//         // dbg!(train_count, total_count, ratio);
//         let index = total_count % (RATIO_DEN * 2);
//         let train = match index {
//             _ if index < TRAIN_RANGE => true,
//             _ if index < VAL_RANGE => false,
//             _ => panic!("Unexpected value"),
//         };

//         if total_count % 1_000 == 0 {
//             print!(
//                 "\r\x1b[K{:.1}% Done!",
//                 100.0 * total_count as f64 / num_entries as f64,
//             );
//             std::io::stdout().flush().unwrap();
//         }

//         if train {
//             train_output.write_all(buff).unwrap();
//             train_count += 1;
//         } else {
//             val_output.write_all(buff).unwrap();
//         }
//     };

//     while let Ok(_) = input.read_exact(&mut buff) {
//         let is_white = match buff[140] {
//             0 => false,
//             1 => true,
//             u => panic!("Got {}", u),
//         };

//         let our_buffer = match is_white {
//             true => &mut white_buff,
//             false => &mut black_buff,
//         };

//         if is_white == was_white_last {
//             our_buffer.push(buff);
//         } else {
//             was_white_last = !was_white_last;
//             write_to_file(&buff, total_count);
//             total_count += 1;
//         }
//     }

//     let zipped = match was_white_last {
//         true => black_buff.iter().zip(white_buff.iter()),
//         false => white_buff.iter().zip(black_buff.iter()),
//     };

//     for (our, their) in zipped {
//         write_to_file(our, total_count);
//         total_count += 1;
//         write_to_file(their, total_count);
//         total_count += 1;
//     }

//     train_output.flush().unwrap();
//     val_output.flush().unwrap();

//     println!(
//         "\r\x1b[K{:.1}% Done!",
//         100.0 * total_count as f64 / num_entries as f64,
//     );
// }
