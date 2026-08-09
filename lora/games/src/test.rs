use std::{
    collections::HashSet,
    str::FromStr,
    sync::{Arc, Mutex},
};

use chess::{Board, Color, MoveGen};
use engine::{
    engine::EVALUATOR,
    evaluation::{evaluator::Evaluator, nnue::NNUEAccumulator},
    Eval,
};
use games::utils::{GameResult, ENTRY_SIZE_BYTES};

use crate::utils::TrainingDataEntry;

pub fn test() {
    let pos = "7r/p4Qpp/5n2/1Bk2RP1/2N4P/1PK5/8/7R b - - 0 1";
    let board = Board::from_str(pos).expect("Invalid board");

    let accumulator = NNUEAccumulator::new(&board, &EVALUATOR);

    // println!("White: {:?}", accumulator[Color::White]);
    // println!("Black: {:?}", accumulator[Color::Black]);
    println!(
        "Eval: {:?}",
        EVALUATOR.eval(&accumulator, board.side_to_move())
    );
    println!("Eval orig: {:?}", Evaluator {}.evaluate(&board));

    let moves = vec![(MoveGen::new_legal(&board).next().unwrap(), Eval::NEUTRAL)];

    let result = GameResult {
        board,
        moves,
        winner: None,
    };

    let data = result.to_bin(Arc::new(Mutex::new(HashSet::new())));

    println!("==================");
    show_data(&data, board.side_to_move());

    // let decoded: TrainingDataEntry = bincode::deserialize(&data).unwrap();

    // println!("{:?}", decoded);
}

fn show_data(data: &[u8], pov: Color) {
    let mut accumulator = NNUEAccumulator::empty(&EVALUATOR);
    println!("Before White: {:?}", accumulator[Color::White]);
    println!("Before Black: {:?}", accumulator[Color::Black]);

    let entry: TrainingDataEntry = bincode::deserialize(&data).unwrap();

    for index in entry.our_features {
        if index == u16::MAX {
            break;
        }
        accumulator.add_feature(index as usize, pov, &EVALUATOR);
    }

    for index in entry.their_features {
        if index == u16::MAX {
            break;
        }
        accumulator.add_feature(index as usize, !pov, &EVALUATOR);
    }

    println!("White: {:?}", accumulator[Color::White]);
    println!("Black: {:?}", accumulator[Color::Black]);

    let eval = EVALUATOR.eval(&accumulator, pov);

    println!("Eval: {:?}", eval);
}
