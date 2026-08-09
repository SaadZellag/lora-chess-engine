use std::{
    collections::HashSet,
    str::FromStr,
    sync::{Arc, Mutex},
};

use chess::{Board, Color, MoveGen};
use engine::{Eval, EVALUATOR, NNUEAccumulator};
use features::FEATURES_PER_SIDE;
use crate::utils::{GameResult, TrainingDataEntry};

pub fn test() {
    let pos = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let board = Board::from_str(pos).expect("Invalid board");

    let accumulator = NNUEAccumulator::new(&board);

    println!(
        "Eval from NNUE: {:?}",
        EVALUATOR.eval(&accumulator, board.side_to_move())
    );

    let moves = vec![(MoveGen::new_legal(&board).next().unwrap(), Eval::CentiPawn(50))];

    let result = GameResult {
        board,
        moves,
        winner: None,
    };

    let data = result.to_bin(Arc::new(Mutex::new(HashSet::new())));

    println!("Generated {} bytes of training data", data.len());
    println!("==================");
    show_data(&data, board.side_to_move());
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
