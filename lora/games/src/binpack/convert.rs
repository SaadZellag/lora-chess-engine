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
        Eval::MateIn(ply) => 32000 + ply as i16,
        Eval::MatedIn(ply) => -32000 + ply as i16,
        Eval::CentiPawn(cp) => cp as i16,
    }
}

pub fn sbin_square_to_chess_square(square: sfbinpack::chess::coords::Square) -> chess::Square {
    let file = match square.file() {
        sfbinpack::chess::coords::File::A => chess::File::A,
        sfbinpack::chess::coords::File::B => chess::File::B,
        sfbinpack::chess::coords::File::C => chess::File::C,
        sfbinpack::chess::coords::File::D => chess::File::D,
        sfbinpack::chess::coords::File::E => chess::File::E,
        sfbinpack::chess::coords::File::F => chess::File::F,
        sfbinpack::chess::coords::File::G => chess::File::G,
        sfbinpack::chess::coords::File::H => chess::File::H,
        _ => unreachable!()
    };

    let rank = match square.rank() {
        sfbinpack::chess::coords::Rank::FIRST => chess::Rank::First,
        sfbinpack::chess::coords::Rank::SECOND => chess::Rank::Second,
        sfbinpack::chess::coords::Rank::THIRD => chess::Rank::Third,
        sfbinpack::chess::coords::Rank::FOURTH => chess::Rank::Fourth,
        sfbinpack::chess::coords::Rank::FIFTH => chess::Rank::Fifth,
        sfbinpack::chess::coords::Rank::SIXTH => chess::Rank::Sixth,
        sfbinpack::chess::coords::Rank::SEVENTH => chess::Rank::Seventh,
        sfbinpack::chess::coords::Rank::EIGHTH => chess::Rank::Eighth,
        _ => unreachable!()
    };

    chess::Square::make_square(rank, file)
}

pub fn chess_square_to_sbin_square(square: chess::Square) -> sfbinpack::chess::coords::Square {
    let file = square.get_file().to_index() as i64;

    let rank = square.get_rank().to_index() as i64;

    sfbinpack::chess::coords::Square::from_rank_file(rank, file)
}

pub fn chess_piece_to_sbin_piece(piece: chess::Piece, color: chess::Color) -> sfbinpack::chess::piece::Piece {
    match (color, piece) {
        (chess::Color::White, chess::Piece::Pawn) => sfbinpack::chess::piece::Piece::WHITE_PAWN,
        (chess::Color::White, chess::Piece::Knight) => sfbinpack::chess::piece::Piece::WHITE_KNIGHT,
        (chess::Color::White, chess::Piece::Bishop) => sfbinpack::chess::piece::Piece::WHITE_BISHOP,
        (chess::Color::White, chess::Piece::Rook) => sfbinpack::chess::piece::Piece::WHITE_ROOK,
        (chess::Color::White, chess::Piece::Queen) => sfbinpack::chess::piece::Piece::WHITE_QUEEN,
        (chess::Color::White, chess::Piece::King) => sfbinpack::chess::piece::Piece::WHITE_KING,
        (chess::Color::Black, chess::Piece::Pawn) => sfbinpack::chess::piece::Piece::BLACK_PAWN,
        (chess::Color::Black, chess::Piece::Knight) => sfbinpack::chess::piece::Piece::BLACK_KNIGHT,
        (chess::Color::Black, chess::Piece::Bishop) => sfbinpack::chess::piece::Piece::BLACK_BISHOP,
        (chess::Color::Black, chess::Piece::Rook) => sfbinpack::chess::piece::Piece::BLACK_ROOK,
        (chess::Color::Black, chess::Piece::Queen) => sfbinpack::chess::piece::Piece::BLACK_QUEEN,
        (chess::Color::Black, chess::Piece::King) => sfbinpack::chess::piece::Piece::BLACK_KING,
    }
}

pub fn sbin_piece_to_chess_piece(piece: sfbinpack::chess::piece::Piece) -> Option<(chess::Piece, chess::Color)> {
    match piece {
        sfbinpack::chess::piece::Piece::WHITE_PAWN => Some((chess::Piece::Pawn, chess::Color::White)),
        sfbinpack::chess::piece::Piece::WHITE_KNIGHT => Some((chess::Piece::Knight, chess::Color::White)),
        sfbinpack::chess::piece::Piece::WHITE_BISHOP => Some((chess::Piece::Bishop, chess::Color::White)),
        sfbinpack::chess::piece::Piece::WHITE_ROOK => Some((chess::Piece::Rook, chess::Color::White)),
        sfbinpack::chess::piece::Piece::WHITE_QUEEN => Some((chess::Piece::Queen, chess::Color::White)),
        sfbinpack::chess::piece::Piece::WHITE_KING => Some((chess::Piece::King, chess::Color::White)),
        sfbinpack::chess::piece::Piece::BLACK_PAWN => Some((chess::Piece::Pawn, chess::Color::Black)),
        sfbinpack::chess::piece::Piece::BLACK_KNIGHT => Some((chess::Piece::Knight, chess::Color::Black)),
        sfbinpack::chess::piece::Piece::BLACK_BISHOP => Some((chess::Piece::Bishop, chess::Color::Black)),
        sfbinpack::chess::piece::Piece::BLACK_ROOK => Some((chess::Piece::Rook, chess::Color::Black)),
        sfbinpack::chess::piece::Piece::BLACK_QUEEN => Some((chess::Piece::Queen, chess::Color::Black)),
        sfbinpack::chess::piece::Piece::BLACK_KING => Some((chess::Piece::King, chess::Color::Black)),
        _ => None,
    }
}