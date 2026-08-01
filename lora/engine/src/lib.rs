use chess::{Board, ChessMove, MoveGen};


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
        Some(moves[0]) // For now, just return the first legal move
    }
}