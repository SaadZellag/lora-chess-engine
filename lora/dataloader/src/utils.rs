use cozy_chess::{Color, Piece};
use features::{FEATURES_PER_SIDE, NUM_FEATURES};
use games::binpack::{GameResult, TrainingEntry};
use stockfish_binpack::{BPEntry, BPResult};


#[derive(Debug)]
#[repr(C)]
pub struct SparseBatch {
    pub size: i32,
    pub num_active_our_features: i32,
    pub num_active_their_features: i32,

    pub final_score: *mut f32,
    pub mom_value: *mut i32,
    pub eval: *mut i32,
    pub our_feature_indices: *mut i32,
    pub their_feature_indices: *mut i32,
}

impl SparseBatch {
    pub fn new(data: &[BPEntry]) -> Self {
        let size = data.len();

        let score: Vec<f32> = data.iter().map(|e| {
            match e.result {
                BPResult::Win => 1.0,
                BPResult::Draw => 0.5,
                BPResult::Loss => 0.0,
            }
        }).collect();

        let mom_value: Vec<i32> = data.iter().map(|e| {
            let board = &e.board;
            let total = 
                board.pieces(Piece::Queen).len() * 9 + 
                board.pieces(Piece::Rook).len() * 5 + 
                board.pieces(Piece::Bishop).len() * 3 + 
                board.pieces(Piece::Knight).len() * 3 + 
                board.pieces(Piece::Pawn).len() * 1;
            total as i32
        }).collect();

        let eval: Vec<i32> = data.iter().map(|e| e.score as i32).collect();

        // dbg!(size);
        // dbg!(data);
        let mut our_feature_indices = vec![0; 2 * size * FEATURES_PER_SIDE];
        let mut their_feature_indices = vec![0; 2 * size * FEATURES_PER_SIDE];

        let mut our_index = 0;
        let mut their_index = 0;

        for (index, item) in data.iter().enumerate() {
            let board_features = features::features(&item.board);

            for (white_feature, black_feature) in board_features {
                debug_assert!(
                    white_feature < NUM_FEATURES && black_feature < NUM_FEATURES,
                    "white_feature is {} and black_feature is {} from index {} | data: {:?}",
                    white_feature,
                    black_feature,
                    index,
                    item
                );

                let (our_feature, their_feature) = match item.board.side_to_move() {
                    Color::White => (white_feature, black_feature),
                    Color::Black => (black_feature, white_feature),
                };

                our_feature_indices[our_index] = index as i32;
                our_feature_indices[our_index + 1] = our_feature as i32;
                our_index += 2;
                their_feature_indices[their_index] = index as i32;
                their_feature_indices[their_index + 1] = their_feature as i32;
                their_index += 2;
            }
        }

        let num_active_white_features = our_index / 2; // Indices contain batch index and value
        let num_active_black_features = their_index / 2;

        Self {
            size: size as i32,
            num_active_our_features: num_active_white_features as i32,
            num_active_their_features: num_active_black_features as i32,
            final_score: score.leak().as_mut_ptr(),
            mom_value: mom_value.leak().as_mut_ptr(),
            eval: eval.leak().as_mut_ptr(),
            our_feature_indices: our_feature_indices.leak().as_mut_ptr(),
            their_feature_indices: their_feature_indices.leak().as_mut_ptr(),
        }
    }
}

impl Drop for SparseBatch {
    fn drop(&mut self) {
        unsafe { 
            let _ = Box::from_raw(self.mom_value);
            let _ = Box::from_raw(self.final_score);
            let _ = Box::from_raw(self.eval);
            let _ = Box::from_raw(self.our_feature_indices);
            let _ = Box::from_raw(self.their_feature_indices);
         }
    }
}
