use chess::Piece;
use chess::{Board, Color, Square};
use std::iter;

const HALFKP_PIECES: [Piece; 5] = [
    Piece::Pawn,
    Piece::Knight,
    Piece::Bishop,
    Piece::Rook,
    Piece::Queen,
];

pub fn white_feature_index(king_sq: Square, piece_sq: Square, piece: Piece, color: Color) -> usize {
    let p_idx = piece.to_index() * 2 + color.to_index();
    piece_sq.to_index() + (p_idx + king_sq.to_index() * 10) * 64
}

pub fn black_feature_index(king_sq: Square, piece_sq: Square, piece: Piece, color: Color) -> usize {
    white_feature_index(flip(king_sq), flip(piece_sq), piece, !color)
}

pub fn features(board: &Board) -> impl Iterator<Item = (usize, usize)> + '_ {
    let white_king_sq = board.king_square(Color::White);
    let black_king_sq = board.king_square(Color::Black);

    HALFKP_PIECES
        .iter()
        .flat_map(|&p| iter::repeat(p).zip(*board.pieces(p)))
        .map(move |(p, sq)| {
            let color = board.color_on(sq).expect("Corrupted board");
            let white_index = white_feature_index(white_king_sq, sq, p, color);
            let black_index = black_feature_index(black_king_sq, sq, p, color);

            (white_index, black_index)
        })
}

#[test]
fn test_halfkp() {
    use chess::{ALL_COLORS, ALL_SQUARES};
    let mut white_result = [0; NUM_FEATURES];
    let mut black_result = [0; NUM_FEATURES];

    for king_sq in ALL_SQUARES {
        for square in ALL_SQUARES {
            for piece in HALFKP_PIECES {
                for color in ALL_COLORS {
                    let index = white_feature_index(king_sq, square, piece, color);
                    white_result[index] += 1;
                    let index = black_feature_index(king_sq, square, piece, color);
                    black_result[index] += 1;
                }
            }
        }
    }

    assert_eq!(white_result, [1; NUM_FEATURES]);
    assert_eq!(black_result, [1; NUM_FEATURES]);
}

fn flip(square: Square) -> Square {
    unsafe { Square::new(square.to_int() ^ 56) }
}
