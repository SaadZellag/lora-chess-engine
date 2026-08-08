use arrayvec::ArrayVec;
use chess::{BitBoard, Board, ChessMove, EMPTY, MoveGen, NUM_PIECES};

use crate::util::positiongen::PositionGenerator;

// victim attacker
// King should NEVER be a victim
const MVV_LVA: [[i32; NUM_PIECES]; NUM_PIECES - 1] = [
    [9, 8, 7, 6, 5, 4],       // Pawn
    [19, 18, 17, 16, 15, 14], // Knight
    [29, 28, 27, 26, 25, 24], // Bishop
    [39, 38, 37, 36, 35, 34], // Rook
    [49, 48, 47, 46, 45, 44], // Queen
];

fn score_move(board: &Board, mv: ChessMove) -> i32 {
    // MVV-LVA
    let attacker = board.piece_on(mv.get_source()).expect("Invalid board");
    let victim = board.piece_on(mv.get_dest());

    if let Some(victim) = victim {
        MVV_LVA[victim as usize][attacker as usize]
    } else {
        0
    }
}

pub struct OrderedMoveGen {
    moves: ArrayVec<ChessMove, 218>,
}

impl OrderedMoveGen {
    pub fn new(board: &Board) -> Self {
        Self::with_mask(board, !chess::EMPTY)
    }

    pub fn with_mask(board: &Board, mask: BitBoard) -> Self {
        let mut itt = MoveGen::new_legal(board);
        itt.set_iterator_mask(mask);

        let mut moves = ArrayVec::new();

        for mv in itt {
            moves.push(mv);
        }

        moves.reverse();

        // Best moves have higher value and we want them last
        // Using pop() to get the moves
        moves.sort_by_cached_key(|mv| score_move(board, *mv));

        Self { moves }
    }

    pub fn remove_move(&mut self, mv: ChessMove) {
        self.moves.retain(|m| m != &mv);
    }

    pub fn allow_only(&mut self, mask: BitBoard) {
        self.moves
            .retain(|mv| BitBoard::from_square(mv.get_dest()) & mask != EMPTY)
    }
}

impl Iterator for OrderedMoveGen {
    type Item = <MoveGen as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.moves.pop()
    }
}

impl ExactSizeIterator for OrderedMoveGen {
    fn len(&self) -> usize {
        self.moves.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::{search::move_ordering::OrderedMoveGen, util::positiongen::PositionGenerator};


    #[test]
    fn test_positions() {
        // To make sure that the ordered movegen generates the same amount of moves as the normal one

        for board in PositionGenerator::new().take(100) {
            let movegen_moves = chess::MoveGen::new_legal(&board);
            let orderedmovegen_moves = OrderedMoveGen::new(&board);

            assert_eq!(movegen_moves.len(), orderedmovegen_moves.len());
            assert_eq!(
                movegen_moves.into_iter().collect::<Vec<_>>().len(),
                orderedmovegen_moves.into_iter().collect::<Vec<_>>().len()
            );
        }
    }
}

// #[test]
// fn test_mvv_lva() {
//     let position =
//         Board::from_str("r3r2k/pbp1q2p/1p6/4n3/2NQ4/2P2pB1/P1P2P1P/2R2RK1 w - - 6 26").unwrap();

//     let gen = OrderedMoveGen::with_mask(&position, *position.combined());

//     for mv in gen {
//         println!("{}", mv);
//     }

//     assert!(false);
// }
