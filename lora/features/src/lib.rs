mod conf;
mod virtual_features;

pub use conf::*;
use cozy_chess::{Board, Color, Piece, Square};

use std::iter;



const HALFKP_PIECES: [Piece; 5] = [
    Piece::Pawn,
    Piece::Knight,
    Piece::Bishop,
    Piece::Rook,
    Piece::Queen,
];

pub fn white_feature_index(king_sq: Square, piece_sq: Square, piece: Piece, color: Color) -> usize {
    let p_idx = piece as usize * 2 + color as usize;
    piece_sq as usize + (p_idx + king_sq as usize * 10) * 64
}

pub fn black_feature_index(king_sq: Square, piece_sq: Square, piece: Piece, color: Color) -> usize {
    white_feature_index(king_sq.flip_rank(), piece_sq.flip_rank(), piece, !color)
}

pub fn features(board: &Board) -> impl Iterator<Item = (usize, usize)> + '_ {
    let white_king_sq = board.king(Color::White);
    let black_king_sq = board.king(Color::Black);

    HALFKP_PIECES
        .iter()
        .flat_map(|&p| iter::repeat(p).zip(board.pieces(p)))
        .map(move |(p, sq)| {
            let color = board.color_on(sq).expect("Corrupted board");
            let white_index = white_feature_index(white_king_sq, sq, p, color);
            let black_index = black_feature_index(black_king_sq, sq, p, color);

            (white_index, black_index)
        })
        // .chain(virtual_features::virtual_features_1(&board))
        // .chain(virtual_features::virtual_features_2(&board))
}


#[cfg(test)]
mod tests {
    use cozy_chess::{Color, Square};

use crate::{HALFKP_PIECES, NUM_FEATURES, black_feature_index, virtual_features, white_feature_index};

    

    #[test]
    fn test_halfkp() {
        let mut white_result = [0; NUM_FEATURES];
        let mut black_result = [0; NUM_FEATURES];

        for king_sq in Square::ALL {
            for square in Square::ALL {
                for piece in HALFKP_PIECES {
                    for color in Color::ALL {
                        let index = white_feature_index(king_sq, square, piece, color);
                        white_result[index] += 1;
                        let index = black_feature_index(king_sq, square, piece, color);
                        black_result[index] += 1;

                        // Virtual features duplicate
                        let index = virtual_features::white_feature_index_virt1(piece, color, square);
                        white_result[index] = 1;
                        let index = virtual_features::black_feature_index_virt1(piece, color, square);
                        black_result[index] = 1;

                        let index = virtual_features::white_feature_index_virt2(piece, color, king_sq);
                        white_result[index] = 1;
                        let index = virtual_features::black_feature_index_virt2(piece, color, king_sq);
                        black_result[index] = 1;
                    }
                }
            }
        }

        assert_eq!(white_result, [1; NUM_FEATURES]);
        assert_eq!(black_result, [1; NUM_FEATURES]);
    }
}
