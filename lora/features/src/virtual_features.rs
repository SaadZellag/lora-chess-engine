use std::iter;

use cozy_chess::{Board, Color, Piece, Square};

use crate::{HALFKP_PIECES, NUM_REGULAR_FEATURES, NUM_VIRTUAL_FEATURES1};

pub fn white_feature_index_virt1(piece: Piece, color: Color, square: Square) -> usize {
    let p_idx = piece as usize * 2 + color as usize;
    NUM_REGULAR_FEATURES + square as usize + p_idx * 64
}

pub fn black_feature_index_virt1(piece: Piece, color: Color, square: Square) -> usize {
    white_feature_index_virt1(piece, !color, square.flip_rank())
}

pub fn white_feature_index_virt2(piece: Piece, color: Color, king_sq: Square) -> usize {
    let p_idx = piece as usize * 2 + color as usize;
    NUM_REGULAR_FEATURES + NUM_VIRTUAL_FEATURES1 + king_sq as usize + p_idx * 64
}

pub fn black_feature_index_virt2(piece: Piece, color: Color, king_sq: Square) -> usize {
    white_feature_index_virt2(piece, !color, king_sq.flip_rank())
}

pub fn virtual_features_1(board: &Board) -> impl Iterator<Item = (usize, usize)> + '_ {
    
    HALFKP_PIECES
        .iter()
        .flat_map(|&p| iter::repeat(p).zip(board.pieces(p)))
        .map(move |(p, sq)| {
            let color = board.color_on(sq).expect("Corrupted board");
            let white_index_virt1 = white_feature_index_virt1(p, color, sq);
            let black_index_virt1 = black_feature_index_virt1(p, color, sq);
            (white_index_virt1, black_index_virt1)
        })
}

pub fn virtual_features_2(board: &Board) -> impl Iterator<Item = (usize, usize)> + '_ {
    
    let white_king_sq = board.king(Color::White);
    let black_king_sq = board.king(Color::Black);

    HALFKP_PIECES
        .iter()
        .flat_map(|&p| iter::repeat(p).zip(board.pieces(p)))
        .map(move |(p, sq)| {
            let color = board.color_on(sq).expect("Corrupted board");
            let white_index_virt2 = white_feature_index_virt2(p, color, white_king_sq);
            let black_index_virt2 = black_feature_index_virt2(p, color, black_king_sq);
            (white_index_virt2, black_index_virt2)
        })
}