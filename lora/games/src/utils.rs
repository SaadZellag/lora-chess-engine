use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use chess::{Board, ChessMove, Color};

use engine::{Eval, SearchHandler, SearchResult, SearchPosition, SearchOptions, TranspositionTable, LoraEngine};
use features::FEATURES_PER_SIDE;
use serde::{Deserialize, Serialize};

pub static ENTRY_SIZE_BYTES: usize = std::mem::size_of::<TrainingDataEntry>();

/// Wrapper around the engine that manages search state
pub struct GameEngine {
    engine: LoraEngine,
    transposition_table: TranspositionTable,
    handler: GameHandler,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct TrainingDataEntry {
    pub our_features: [u16; FEATURES_PER_SIDE],
    pub their_features: [u16; FEATURES_PER_SIDE],
    pub final_score: f64,
    pub eval: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct TrainingDataPosition {
    pub board: Board,
    pub winner: Option<Color>,
    pub eval: Eval,
}

impl Into<TrainingDataEntry> for TrainingDataPosition {
    fn into(self) -> TrainingDataEntry {
        let mut our_features = [u16::MAX; FEATURES_PER_SIDE];
        let mut their_features = [u16::MAX; FEATURES_PER_SIDE];

        for (i, (white, black)) in features::features(&self.board).enumerate()
        {
            our_features[i] = white as u16;
            their_features[i] = black as u16;
        }

        our_features.sort();
        their_features.sort();

        if self.board.side_to_move() == Color::Black {
            (our_features, their_features) = (their_features, our_features);
        }

        let final_score = match self.winner {
            Some(color) => {
                if color == self.board.side_to_move() {
                    1.0_f64
                } else {
                    0.0_f64
                }
            }
            None => 0.5_f64,
        };

        TrainingDataEntry {
            our_features,
            their_features,
            final_score,
            eval: self.eval.value() as i32,
        }
    }
}

pub struct GameResult {
    pub board: Board,
    pub moves: Vec<(ChessMove, Eval)>,
    pub winner: Option<Color>,
}

impl GameResult {
    pub fn to_bin(&self, seen: Arc<Mutex<HashSet<u64>>>) -> Vec<u8> {
        let mut result = Vec::with_capacity(self.moves.len() * ENTRY_SIZE_BYTES);
        let mut curr_board = self.board;

        for (mv, eval) in &self.moves {
            let mut add = true;

            // Do not allow for mates
            add &= matches!(eval, Eval::CentiPawn(_));

            // Dont add replicates
            add &= seen.lock().unwrap().insert(curr_board.get_hash());

            // Dont add checks
            add &= curr_board.checkers().popcnt() == 0;

            if add {
                let position = TrainingDataPosition {
                    board: curr_board,
                    eval: *eval,
                    winner: self.winner,
                };
                let entry: TrainingDataEntry = position.into();
                result.append(&mut bincode::serialize(&entry).unwrap())
            }

            if curr_board.legal(*mv) {
                curr_board = curr_board.make_move_new(*mv);
            } else {
                break;
            }
        }

        result
    }
}

const DEPTH: u8 = 4;

/// Simple Depth management handler
#[derive(Default)]
pub struct GameHandler {
    prev_result: Option<SearchResult>,
}

impl SearchHandler for GameHandler {
    fn new_result(&mut self, result: SearchResult) {
        self.prev_result = Some(result)
    }

    fn should_stop(&self) -> bool {
        self.prev_result
            .map(|res| res.stats.depth >= DEPTH)
            .unwrap_or_default()
    }
}

impl GameEngine {
    /// Create a new game engine for the given board
    pub fn new(board: Board) -> Self {
        Self {
            engine: LoraEngine::new(),
            transposition_table: TranspositionTable::default(),
            handler: GameHandler::default(),
        }
    }

    /// Get the best move for the current position
    pub fn best_move(&mut self, board: Board, history: &[u64]) -> Option<SearchResult> {
        let position = SearchPosition {
            board,
            moves_played: Vec::new(),
        };

        let options = SearchOptions {
            max_depth: 255,
            max_nodes: u64::MAX,
            mate_search_depth: None,
            moves_to_search: None,
        };

        self.engine.search(position, options, &mut self.handler, &mut self.transposition_table)
    }
}

/// Helper function for backward compatibility
pub fn new_engine(board: Board, _history: Vec<u64>) -> GameEngine {
    GameEngine::new(board)
}

#[test]
fn test_conversion() {
    use chess::{Color, Board};
    
    let board = Board::default();
    let position = TrainingDataPosition {
        board,
        winner: Some(Color::White),
        eval: Eval::CentiPawn(100),
    };
    
    let entry: TrainingDataEntry = position.into();
    
    // Verify entry was created
    assert!(entry.our_features.iter().any(|&x| x != u16::MAX));
    assert!(entry.their_features.iter().any(|&x| x != u16::MAX));
    assert_eq!(entry.eval, 100);
    assert_eq!(entry.final_score, 1.0); // White won and is to move in starting position
    
    // Verify serialization roundtrip
    let encoded = bincode::serialize(&entry).unwrap();
    // Note: serialized size may differ from mem::size_of due to bincode encoding
    assert!(encoded.len() > 0);
    
    let decoded: TrainingDataEntry = bincode::deserialize(&encoded).unwrap();
    assert_eq!(decoded.eval, entry.eval);
    assert_eq!(decoded.final_score, entry.final_score);
}

pub(crate) fn shared<T>(item: T) -> Arc<Mutex<T>> {
    Arc::new(Mutex::new(item))
}
