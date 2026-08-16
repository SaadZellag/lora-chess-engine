use arrayvec::ArrayVec;
use cozy_chess::{BitBoard, Board, Move};

const MAX_LEGAL_MOVES: usize = 218;


pub trait CozyChessHelper {
    fn legal_moves(&self) -> ArrayVec<Move, MAX_LEGAL_MOVES> {
        self.legal_moves_with_mask(BitBoard::FULL)
    }
    fn legal_moves_with_mask(&self, mask: BitBoard) -> ArrayVec<Move, MAX_LEGAL_MOVES>;

    fn play_unchecked_new(&self, mv: Move) -> Self;
}

impl CozyChessHelper for Board {
    fn legal_moves_with_mask(&self, mask: BitBoard) -> ArrayVec<Move, MAX_LEGAL_MOVES> {
        let mut moves = ArrayVec::new();
        self.generate_moves_for(mask, |legal_moves| {
            for mv in legal_moves {
                moves.push(mv);
            }
            false
        });
        moves
    }
    fn play_unchecked_new(&self, mv: Move) -> Self {
        let mut new_board = self.clone();
        new_board.play_unchecked(mv);
        new_board
    }
}