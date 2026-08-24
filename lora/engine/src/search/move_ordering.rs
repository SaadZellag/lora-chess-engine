use arrayvec::ArrayVec;
use cozy_chess::{BitBoard, Board, Move, Piece};

use crate::util::positiongen::PositionGenerator;

// victim attacker
// King should NEVER be a victim
const MVV_LVA: [[i32; Piece::NUM]; Piece::NUM - 1] = [
    [9, 8, 7, 6, 5, 4],       // Pawn
    [19, 18, 17, 16, 15, 14], // Knight
    [29, 28, 27, 26, 25, 24], // Bishop
    [39, 38, 37, 36, 35, 34], // Rook
    [49, 48, 47, 46, 45, 44], // Queen
];

fn score_move(board: &Board, mv: Move) -> i32 {
    // MVV-LVA
    let attacker = board.piece_on(mv.from).expect("Invalid board");
    let victim = board.piece_on(mv.to);

    if let Some(victim) = victim {
        MVV_LVA[victim as usize][attacker as usize]
    } else {
        0
    }
}

pub struct OrderedMoveGen {
    moves: ArrayVec<Move, 218>,
}

impl OrderedMoveGen {
    pub fn new(board: &Board) -> Self {
        Self::with_mask(board, !BitBoard::EMPTY)
    }

    pub fn with_mask(board: &Board, mask: BitBoard) -> Self {
        let mut all_moves = ArrayVec::new();

        board.generate_moves(|mut moves| {
            moves.to = moves.to & mask;
            for _mv in moves {
                all_moves.push(_mv);
            }
            false
        });

        // Best moves have higher value and we want them last
        // Using pop() to get the moves
        all_moves.sort_by_cached_key(|mv| score_move(board, *mv));

        Self { moves: all_moves }
    }

    pub fn remove_move(&mut self, mv: Move) {
        self.moves.retain(|m| m != &mv);
    }

    pub fn allow_only(&mut self, mask: BitBoard) {

        self.moves
            .retain(|mv| mv.to.bitboard() & mask != BitBoard::EMPTY)
    }
}

impl Iterator for OrderedMoveGen {
    type Item = Move;

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
            let mut movegen_moves = vec![];
            board.generate_moves(|moves| {
                for _mv in moves {
                    movegen_moves.push(_mv);
                }
                false
            });
            let orderedmovegen_moves = OrderedMoveGen::new(&board);

            assert_eq!(movegen_moves.len(), orderedmovegen_moves.len());
            assert_eq!(
                movegen_moves.len(),
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
