mod eval;
pub mod util;
mod search;

pub use eval::Eval;
pub use search::{SearchHandler, SearchPosition, SearchResult, SearchStats, SearchOptions};
pub use search::tt::TranspositionTable;
pub use eval::nnue::{EVALUATOR, NNUEAccumulator, NNUE};
pub use eval::nnue::nnue_conf;

pub const MAX_DEPTH: u8 = u8::MAX;

pub struct LoraEngine {
    pub options: EngineOptions,
}

#[derive(Debug, Clone, Copy)]
pub struct EngineOptions {
    pub num_threads: usize,
    pub tt_size_bytes: usize,
}
impl LoraEngine {
    pub fn new() -> Self {
        LoraEngine {
            options: EngineOptions {
                num_threads: 1,
                tt_size_bytes: 1024 * 1024, // 1 MB
            }
        }
    }

    pub fn set_hash_size(&mut self, size_mb: usize) {
        self.options.tt_size_bytes = size_mb * 1024 * 1024;
    }
}

impl Default for LoraEngine {
    fn default() -> Self {
        Self::new()
    }
}
