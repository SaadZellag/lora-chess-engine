use crate::{
    Eval,
    eval::nnue::{
        EVALUATOR, NNUE, NNUEAccumulator,
        activations::CRELU,
        vectors::{fast_vadd, fast_vsub},
    },
    nnue_conf::L1,
};
use common::cozy_chess::CozyChessHelper;
use cozy_chess::{Board, Move, Color, Piece, Square};
use std::ops::{Index, IndexMut};

impl Index<Color> for NNUEAccumulator {
    type Output = [i16; L1];

    fn index(&self, index: Color) -> &Self::Output {
        match index {
            Color::White => &self.v[0],
            Color::Black => &self.v[1],
        }
    }
}

impl IndexMut<Color> for NNUEAccumulator {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        match index {
            Color::White => &mut self.v[0],
            Color::Black => &mut self.v[1],
        }
    }
}

impl NNUE {
    pub fn eval(&self, acc: &NNUEAccumulator, stm: Color) -> Eval {
        let mut input = [0; { L1 * 2 }];

        for i in 0..L1 {
            input[i] = acc[stm][i].into();
        }

        for i in 0..L1 {
            input[L1 + i] = acc[!stm][i].into();
        }

        let layer_1_out = self.layer_1.activate(&input.crelu(1));
        // let layer_2_out = self.layer_2.activate(&layer_1_out.crelu(64));
        let output = self.output.activate(&layer_1_out.crelu(64));

        Eval::CentiPawn(output[0] / 8)
    }

    pub fn show(&self) {
        println!("{:?}", self.ft)
    }
}

impl NNUEAccumulator {
    pub fn new(board: &Board) -> Self {
        let mut result = Self::empty();

        let features = features::features(board);

        for (white, black) in features {
            result.add_feature(white, Color::White);
            result.add_feature(black, Color::Black);
        }

        result
    }

    pub fn empty() -> Self {
        let mut result = Self { v: [[0; L1]; 2] };

        for i in 0..L1 {
            result[Color::White][i] = EVALUATOR.ft.bias[i];
            result[Color::Black][i] = EVALUATOR.ft.bias[i];
        }

        result
    }

    pub fn add_feature(&mut self, index: usize, color: Color) {
        self[color] = fast_vadd(&self[color], &EVALUATOR.ft.weights[index]);
    }

    pub fn remove_feature(&mut self, index: usize, color: Color) {
        self[color] = fast_vsub(&self[color], &EVALUATOR.ft.weights[index]);
    }

    pub fn update(&self, initial_board: &Board, final_board: &Board, mv: Move) -> Self {
        debug_assert_eq!(&initial_board.play_unchecked_new(mv), final_board);

        let piece_moving = initial_board
            .piece_on(mv.from)
            .expect("Invalid move for board");

        // Since our feature set may be halfkp, any move by the king may
        // reset every single feature, it only becomes problematic in the endgame
        // where the king moves a lot, but the number of pieces aren't many
        // TODO: Only the side moving should refresh, if white moves king then black shouldn't have to refresh
        // Optimization left for later
        if piece_moving == Piece::King {
            return Self::new(final_board);
        }

        let mut result = *self;

        let white_king = initial_board.king(Color::White);
        let black_king = initial_board.king(Color::Black);

        let from = mv.from;
        let to = mv.to;
        let stm = initial_board.side_to_move();

        let piece_captured = initial_board.piece_on(to);

        // Check if there is capture
        if let Some(captured) = piece_captured {
            let captured_color = initial_board.color_on(to).unwrap(); // Should always unwrap
            let white_index =
                features::white_feature_index(white_king, to, captured, captured_color);
            let black_index =
                features::black_feature_index(black_king, to, captured, captured_color);

            result.remove_feature(white_index, Color::White);
            result.remove_feature(black_index, Color::Black);
        }

        // Removing from square
        let white_index = features::white_feature_index(white_king, from, piece_moving, stm);
        let black_index = features::black_feature_index(black_king, from, piece_moving, stm);

        result.remove_feature(white_index, Color::White);
        result.remove_feature(black_index, Color::Black);

        // Promotion
        let piece_at_destination = match mv.promotion {
            Some(piece) => piece,
            None => piece_moving,
        };

        // Adding to square
        let white_index = features::white_feature_index(white_king, to, piece_at_destination, stm);
        let black_index = features::black_feature_index(black_king, to, piece_at_destination, stm);

        result.add_feature(white_index, Color::White);
        result.add_feature(black_index, Color::Black);

        // En Passant
        // If pawn moved in diagonal and it ate nothing
        let en_passant = piece_moving == Piece::Pawn
            && from.file() != to.file()
            && piece_captured.is_none();

        if en_passant {
            let square = Square::new(to.file(), from.rank());
            // Removing pawn
            // Adding to square
            let white_index = features::white_feature_index(white_king, square, Piece::Pawn, !stm);
            let black_index = features::black_feature_index(black_king, square, Piece::Pawn, !stm);

            result.remove_feature(white_index, Color::White);
            result.remove_feature(black_index, Color::Black);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use common::cozy_chess::CozyChessHelper;

    use crate::{eval::nnue::NNUEAccumulator, util::positiongen::PositionGenerator};

    #[test]
    fn test_acc_update() {
        for board in PositionGenerator::new().take(1000) {
            let acc = NNUEAccumulator::new(&board);

            for mv in board.legal_moves() {
                let new_board = board.play_unchecked_new(mv);
                let new_acc = acc.update(&board, &new_board, mv);

                assert_eq!(
                    new_acc,
                    NNUEAccumulator::new(&new_board),
                    "{} with {} played doesn't match",
                    board,
                    mv
                );
            }
        }
    }
}
