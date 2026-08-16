use engine::Eval;


pub fn stockfish_eval_to_eval(score: i16) -> Eval {

    match score {
        -32000..=-31000 => return Eval::MatedIn((score + 32000).try_into().unwrap()),
        31000..=32000 => return Eval::MateIn((32000 - score).try_into().unwrap()),
        eval => Eval::CentiPawn(eval.into())
    }

}

pub fn eval_to_stockfish_eval(eval: Eval) -> i16 {
    match eval {
        Eval::MateIn(ply) => 32000 - ply as i16,
        Eval::MatedIn(ply) => -32000 + ply as i16,
        Eval::CentiPawn(cp) => cp as i16,
    }
}

pub fn sbin_square_to_chess_square(square: sfbinpack::chess::coords::Square) -> cozy_chess::Square {
    let file = match square.file() {
        sfbinpack::chess::coords::File::A => cozy_chess::File::A,
        sfbinpack::chess::coords::File::B => cozy_chess::File::B,
        sfbinpack::chess::coords::File::C => cozy_chess::File::C,
        sfbinpack::chess::coords::File::D => cozy_chess::File::D,
        sfbinpack::chess::coords::File::E => cozy_chess::File::E,
        sfbinpack::chess::coords::File::F => cozy_chess::File::F,
        sfbinpack::chess::coords::File::G => cozy_chess::File::G,
        sfbinpack::chess::coords::File::H => cozy_chess::File::H,
        _ => unreachable!()
    };

    let rank = match square.rank() {
        sfbinpack::chess::coords::Rank::FIRST => cozy_chess::Rank::First,
        sfbinpack::chess::coords::Rank::SECOND => cozy_chess::Rank::Second,
        sfbinpack::chess::coords::Rank::THIRD => cozy_chess::Rank::Third,
        sfbinpack::chess::coords::Rank::FOURTH => cozy_chess::Rank::Fourth,
        sfbinpack::chess::coords::Rank::FIFTH => cozy_chess::Rank::Fifth,
        sfbinpack::chess::coords::Rank::SIXTH => cozy_chess::Rank::Sixth,
        sfbinpack::chess::coords::Rank::SEVENTH => cozy_chess::Rank::Seventh,
        sfbinpack::chess::coords::Rank::EIGHTH => cozy_chess::Rank::Eighth,
        _ => unreachable!()
    };

    cozy_chess::Square::new(file, rank)
}

pub fn chess_square_to_sbin_square(square: cozy_chess::Square) -> sfbinpack::chess::coords::Square {
    let file = square.file() as i64;

    let rank = square.rank() as i64;

    sfbinpack::chess::coords::Square::from_rank_file(rank, file)
}

pub fn chess_piece_to_sbin_piece(piece: cozy_chess::Piece, color: cozy_chess::Color) -> sfbinpack::chess::piece::Piece {
    match (color, piece) {
        (cozy_chess::Color::White, cozy_chess::Piece::Pawn) => sfbinpack::chess::piece::Piece::WHITE_PAWN,
        (cozy_chess::Color::White, cozy_chess::Piece::Knight) => sfbinpack::chess::piece::Piece::WHITE_KNIGHT,
        (cozy_chess::Color::White, cozy_chess::Piece::Bishop) => sfbinpack::chess::piece::Piece::WHITE_BISHOP,
        (cozy_chess::Color::White, cozy_chess::Piece::Rook) => sfbinpack::chess::piece::Piece::WHITE_ROOK,
        (cozy_chess::Color::White, cozy_chess::Piece::Queen) => sfbinpack::chess::piece::Piece::WHITE_QUEEN,
        (cozy_chess::Color::White, cozy_chess::Piece::King) => sfbinpack::chess::piece::Piece::WHITE_KING,
        (cozy_chess::Color::Black, cozy_chess::Piece::Pawn) => sfbinpack::chess::piece::Piece::BLACK_PAWN,
        (cozy_chess::Color::Black, cozy_chess::Piece::Knight) => sfbinpack::chess::piece::Piece::BLACK_KNIGHT,
        (cozy_chess::Color::Black, cozy_chess::Piece::Bishop) => sfbinpack::chess::piece::Piece::BLACK_BISHOP,
        (cozy_chess::Color::Black, cozy_chess::Piece::Rook) => sfbinpack::chess::piece::Piece::BLACK_ROOK,
        (cozy_chess::Color::Black, cozy_chess::Piece::Queen) => sfbinpack::chess::piece::Piece::BLACK_QUEEN,
        (cozy_chess::Color::Black, cozy_chess::Piece::King) => sfbinpack::chess::piece::Piece::BLACK_KING,
    }
}

pub fn sbin_piece_to_chess_piece(piece: sfbinpack::chess::piece::Piece) -> Option<(cozy_chess::Piece, cozy_chess::Color)> {
    match piece {
        sfbinpack::chess::piece::Piece::WHITE_PAWN => Some((cozy_chess::Piece::Pawn, cozy_chess::Color::White)),
        sfbinpack::chess::piece::Piece::WHITE_KNIGHT => Some((cozy_chess::Piece::Knight, cozy_chess::Color::White)),
        sfbinpack::chess::piece::Piece::WHITE_BISHOP => Some((cozy_chess::Piece::Bishop, cozy_chess::Color::White)),
        sfbinpack::chess::piece::Piece::WHITE_ROOK => Some((cozy_chess::Piece::Rook, cozy_chess::Color::White)),
        sfbinpack::chess::piece::Piece::WHITE_QUEEN => Some((cozy_chess::Piece::Queen, cozy_chess::Color::White)),
        sfbinpack::chess::piece::Piece::WHITE_KING => Some((cozy_chess::Piece::King, cozy_chess::Color::White)),
        sfbinpack::chess::piece::Piece::BLACK_PAWN => Some((cozy_chess::Piece::Pawn, cozy_chess::Color::Black)),
        sfbinpack::chess::piece::Piece::BLACK_KNIGHT => Some((cozy_chess::Piece::Knight, cozy_chess::Color::Black)),
        sfbinpack::chess::piece::Piece::BLACK_BISHOP => Some((cozy_chess::Piece::Bishop, cozy_chess::Color::Black)),
        sfbinpack::chess::piece::Piece::BLACK_ROOK => Some((cozy_chess::Piece::Rook, cozy_chess::Color::Black)),
        sfbinpack::chess::piece::Piece::BLACK_QUEEN => Some((cozy_chess::Piece::Queen, cozy_chess::Color::Black)),
        sfbinpack::chess::piece::Piece::BLACK_KING => Some((cozy_chess::Piece::King, cozy_chess::Color::Black)),
        _ => None,
    }
}