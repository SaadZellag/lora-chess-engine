pub mod position;
pub mod tt;
mod impls;
mod move_ordering;
mod quiese;

use std::rc::Rc;

use cozy_chess::{Board, Move, Piece};

use crate::{EngineOptions, Eval, LoraEngine, search::tt::TranspositionTable, MAX_DEPTH};

pub trait SearchHandler {
    fn new_result(&mut self, result: SearchResult);

    fn should_stop(&self) -> bool;
}

#[derive(Debug, Clone)]
pub struct SearchPosition {
    pub board: Board,
    pub moves_played: Vec<Move>,
}

#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub max_depth: u8,
    pub max_nodes: u64,
    pub mate_search_depth: Option<u8>,
    pub moves_to_search: Option<Vec<Move>>,
}

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub best_move: Move,
    pub eval: Eval,
    pub stats: SearchStats,
    pub hashfull: usize,
    pub pv: Vec<Move>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SearchStats {
    pub nodes_visited: u64,
    pub depth: u8,
    pub sel_depth: u8,
    pub tbl_hits: u64,
}

struct EngineSearcher<'a, H: SearchHandler> {
    engine_options: EngineOptions,
    position: SearchPosition,
    search_options: SearchOptions,
    search_state: SearchState,
    handler: &'a mut H,
    transposition_table: &'a mut TranspositionTable,
}

struct SearchState {
    pub nodes_searched: u64,
    pub depth_reached: u8,
    pub selective_depth_reached: u8,
    pub history_hash: Vec<u64>,
    pub table_hits: u64,
}



impl LoraEngine {
    pub fn search<H: SearchHandler>(&self, position: SearchPosition, search_options: SearchOptions, handler: &mut H, tt: &mut TranspositionTable) -> Option<SearchResult> {
        let mut searcher = EngineSearcher::new(position, self.options, search_options, handler, tt);
        searcher.best_move()
    }
}
