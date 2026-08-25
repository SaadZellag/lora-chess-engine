pub const FEATURES_PER_SIDE: usize = 30 * 3; // regular features, piece_pos, relative_to_king

pub const NUM_REGULAR_FEATURES: usize = 64 * 64 * 5 * 2; // king_sq, piece_sq, piece, color
pub const NUM_VIRTUAL_FEATURES1: usize = 64 * 10; // piece_pos
pub const NUM_VIRTUAL_FEATURES2: usize = 64 * 10; // relative_to_king

pub const NUM_FEATURES: usize = NUM_REGULAR_FEATURES + NUM_VIRTUAL_FEATURES1 + NUM_VIRTUAL_FEATURES2;
