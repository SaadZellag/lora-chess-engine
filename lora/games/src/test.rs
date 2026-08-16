use std::{
    collections::HashSet,
    str::FromStr,
    sync::{Arc, Mutex},
};

use common::cozy_chess::CozyChessHelper;
use cozy_chess::{Board, Color};
use engine::{Eval, EVALUATOR, NNUEAccumulator};
use features::FEATURES_PER_SIDE;
use games::binpack::{BinPackReader, BinPackWriter};
use indicatif::ProgressStyle;
use tempfile::tempdir;
use crate::utils::{GameResult, TrainingDataEntry};

pub fn test(binpack_file: &str) {
    let pos = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let board = Board::from_str(pos).expect("Invalid board");

    let accumulator = NNUEAccumulator::new(&board);

    println!(
        "Eval from NNUE: {:?}",
        EVALUATOR.eval(&accumulator, board.side_to_move())
    );

    let moves = vec![(board.legal_moves().pop().unwrap(), Eval::CentiPawn(50))];

    let result = GameResult {
        board: board.clone(),
        moves,
        winner: None,
    };

    let data = result.to_bin(Arc::new(Mutex::new(HashSet::new())));

    println!("Generated {} bytes of training data", data.len());
    println!("==================");
    show_data(&data, board.side_to_move());

    test_binpack_read_write(binpack_file);
}

fn show_data(data: &[u8], pov: Color) {
    // The actual serialized size might be different from ENTRY_SIZE_BYTES (which is mem::size_of)
    let serialized_size = bincode::serialized_size(&TrainingDataEntry {
        our_features: [0; FEATURES_PER_SIDE],
        their_features: [0; FEATURES_PER_SIDE],
        final_score: 0.0,
        eval: 0,
    }).unwrap() as usize;
    
    if data.len() < serialized_size {
        println!("Data too small: {} bytes (expected at least {})", data.len(), serialized_size);
        return;
    }

    let mut accumulator = NNUEAccumulator::empty();

    let entry: TrainingDataEntry = bincode::deserialize(&data[..serialized_size]).unwrap();

    println!("Before reconstruction:");
    println!("White acc sample: {:?}", accumulator[Color::White][..8].to_vec());
    println!("Black acc sample: {:?}", accumulator[Color::Black][..8].to_vec());

    // Reconstruct features from entry
    for index in entry.our_features {
        if index == u16::MAX {
            break;
        }
        accumulator.add_feature(index as usize, pov);
    }

    for index in entry.their_features {
        if index == u16::MAX {
            break;
        }
        accumulator.add_feature(index as usize, !pov);
    }

    println!("After reconstruction:");
    println!("White acc sample: {:?}", accumulator[Color::White][..8].to_vec());
    println!("Black acc sample: {:?}", accumulator[Color::Black][..8].to_vec());

    let eval = EVALUATOR.eval(&accumulator, pov);

    println!("Eval from reconstructed accumulator: {:?}", eval);
}

fn test_binpack_read_write(binpack_file: &str) {
    let file = std::fs::File::open(binpack_file).expect("Failed to open binpack file");

    let file_size = file.metadata().expect("Failed to get file metadata").len();
    let expected_entries = file_size * 2 / 5;

    let dir = tempdir().unwrap();
    let path = dir.path().join("test_file.binpack");
    let output_file = std::fs::File::create(&path).expect("Failed to create test file");

    let mut reader = BinPackReader::new(file).expect("Failed to create BinPackReader");
    let mut writer = BinPackWriter::new(output_file).expect("Failed to create BinPackWriter");

    let progress_bar = indicatif::ProgressBar::new(expected_entries);
    progress_bar.set_style(ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}")
    .unwrap()
    .progress_chars("##-"));
    

    println!("Test file: {:?}", path);

    let chunk_size = 1000;

    while let Ok(entries) = reader.get_next_entries(chunk_size) {
        for entry in &entries {
            writer.write_entry(&entry).expect("Failed to write entry");
        }
        progress_bar.inc(entries.len() as u64);
        progress_bar.set_message(format!("ETA: {:?}", progress_bar.eta()));
    }

    progress_bar.finish_with_message("Done reading and writing entries");
}