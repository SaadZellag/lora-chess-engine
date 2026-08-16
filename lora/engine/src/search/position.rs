use crate::{
    Eval, eval::nnue::{EVALUATOR, NNUEAccumulator}, search::move_ordering::OrderedMoveGen,
};
use common::cozy_chess::CozyChessHelper;
use cozy_chess::{Board, GameStatus, Move};

#[derive(Debug, Clone)]
pub struct Position {
    board: Board,
    acc: NNUEAccumulator,
    ply: u8,
}

impl Position {
    pub fn new(board: Board) -> Self {
        let acc = NNUEAccumulator::new(&board);
        Self { board, acc, ply: 0 }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn acc(&self) -> &NNUEAccumulator {
        &self.acc
    }

    pub fn ply(&self) -> u8 {
        self.ply
    }

    pub fn possible_moves(&self) -> OrderedMoveGen {
        OrderedMoveGen::new(&self.board)
    }

    pub fn possible_captures(&self) -> OrderedMoveGen {
        OrderedMoveGen::with_mask(&self.board, self.board.occupied())
    }

    pub fn make_move(&self, mv: Move) -> Self {
        let new_board = self.board.play_unchecked_new(mv);

        let acc = self.acc.update(&self.board, &new_board, mv);
        Self {
            board: new_board,
            acc,
            ply: self.ply + 1,
        }
    }

    pub fn null_move(&self) -> Option<Self> {
        Some(Self {
            board: self.board.null_move()?,
            acc: self.acc,
            ply: self.ply + 1,
        })
    }

    pub fn eval(&self) -> Eval {
        match self.board.status() {
            GameStatus::Ongoing => {}
            GameStatus::Drawn => return Eval::NEUTRAL,
            GameStatus::Won => return Eval::MatedIn(self.ply.into()),
        };

        EVALUATOR.eval(&self.acc, self.board.side_to_move())
    }
}
