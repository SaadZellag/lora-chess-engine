mod eval;
pub mod util;
mod search;

pub use eval::Eval;
pub use search::{SearchHandler, SearchPosition, SearchResult, SearchStats, SearchOptions};

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
}

impl Default for LoraEngine {
    fn default() -> Self {
        Self::new()
    }
}
