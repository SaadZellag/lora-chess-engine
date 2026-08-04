mod eval;
mod search;
mod tt;
mod util;

pub use eval::Eval;

use chess::ChessMove;

use crate::tt::TranspositionTable;

pub const MAX_DEPTH: u8 = u8::MAX;

pub struct LoraEngine {
    pub options: EngineOptions,
    pub state: SearchState,
}

#[derive(Debug, Clone)]
pub struct EngineOptions {
    pub tt_size_bytes: usize,
    pub max_depth: u8,
    pub max_nodes: u64,
    pub mate_search_depth: Option<u8>,
    pub moves_to_search: Option<Vec<ChessMove>>,
}

pub struct SearchState {
    pub nodes_searched: u64,
    pub depth_reached: u8,
    pub selective_depth_reached: u8,
    pub history_hash: Vec<u64>,
    pub transposition_table: TranspositionTable,
}

impl LoraEngine {
    pub fn new() -> Self {
        LoraEngine {
            options: EngineOptions {
                tt_size_bytes: 1024,
                max_depth: MAX_DEPTH,
                max_nodes: u64::MAX,
                mate_search_depth: None,
                moves_to_search: None,
            },
            state: SearchState {
                nodes_searched: 0,
                depth_reached: 0,
                selective_depth_reached: 0,
                history_hash: Vec::new(),
                transposition_table: TranspositionTable::new(1024),
            },
        }
    }
}

impl Default for LoraEngine {
    fn default() -> Self {
        Self::new()
    }
}
