use chess::{Board, ChessMove, MoveGen, Piece};


pub struct LoraEngine {

}

impl LoraEngine {
    pub fn new() -> Self {
        LoraEngine {

        }
    }

    pub fn search(&mut self, board: &Board) -> Option<ChessMove> {
        let movegen = MoveGen::new_legal(board);
        let moves = movegen.collect::<Vec<ChessMove>>();
        if moves.is_empty() {
            return None;
        }
        
        // Get the best move based on the highest piece value captured
        let best_move = moves.into_iter().max_by_key(|mv| {
            let captured_piece = board.piece_on(mv.get_dest());
            match captured_piece {
                None => 0,
                Some(Piece::Pawn) => 1,
                Some(Piece::Knight) => 3,
                Some(Piece::Bishop) => 3,
                Some(Piece::Rook) => 5,
                Some(Piece::Queen) => 9,
                Some(Piece::King) => 1000,
            }
        });
        best_move
    }
}