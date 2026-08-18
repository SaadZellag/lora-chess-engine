use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use cozy_chess::{Board, Move, Color};

use engine::{Eval, SearchHandler, SearchResult, SearchPosition, SearchOptions, TranspositionTable, LoraEngine};
use features::FEATURES_PER_SIDE;
use serde::{Deserialize, Serialize};

/// Wrapper around the engine that manages search state
pub struct GameEngine {
    engine: LoraEngine,
    transposition_table: TranspositionTable,
    handler: GameHandler,
}

/// Node-based search management handler
#[derive(Default, Debug)]
pub struct GameHandler {
    prev_result: Option<SearchResult>,
    max_nodes: u64,
}

impl GameHandler {
    pub fn new(max_nodes: usize) -> Self {
        Self {
            prev_result: None,
            max_nodes: max_nodes as u64,
        }
    }

    pub fn reset(&mut self) {
        self.prev_result = None;
    }
}

impl SearchHandler for GameHandler {
    fn new_result(&mut self, result: SearchResult) {
        self.prev_result = Some(result)
    }

    fn should_stop(&self) -> bool {
        self.prev_result.as_ref()
            .map(|res| res.stats.nodes_visited >= self.max_nodes)
            .unwrap_or_default()
    }
}

impl GameEngine {
    /// Create a new game engine for the given board
    pub fn new(_board: Board) -> Self {
        Self {
            engine: LoraEngine::new(),
            transposition_table: TranspositionTable::default(),
            handler: GameHandler::default(),
        }
    }

    /// Create a new game engine with specified node limit
    pub fn with_nodes(max_nodes: usize) -> Self {
        Self {
            engine: LoraEngine::new(),
            transposition_table: TranspositionTable::default(),
            handler: GameHandler::new(max_nodes),
        }
    }

    /// Get the best move for the current position
    pub fn best_move(&mut self, board: Board, _history: &[u64]) -> Option<SearchResult> {
        // Reset handler state for this new search
        self.handler.reset();

        let position = SearchPosition {
            board,
            moves_played: Vec::new(),
        };

        let options = SearchOptions {
            max_depth: 255,
            max_nodes: self.handler.max_nodes,
            mate_search_depth: None,
            moves_to_search: None,
        };

        self.engine.search(position, options, &mut self.handler, &mut self.transposition_table)
    }
}

/// Helper function to create engine with specific node limit
pub fn new_engine(nodes: usize) -> GameEngine {
    GameEngine::with_nodes(nodes)
}


pub(crate) fn shared<T>(item: T) -> Arc<Mutex<T>> {
    Arc::new(Mutex::new(item))
}
