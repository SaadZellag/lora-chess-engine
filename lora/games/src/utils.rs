use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use chess::{Board, ChessMove, Color};

use engine::{Eval, SearchHandler, SearchResult};
use features::FEATURES_PER_SIDE;
use serde::{Deserialize, Serialize};

pub static ENTRY_SIZE_BYTES: usize = std::mem::size_of::<TrainingDataEntry>();

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
// Simple Depth management handler
#[derive(Default)]
pub struct GameHandler {
    prev_result: Option<SearchResult>,
}

impl GameHandler {
    fn new() -> Self {
        Self { prev_result: None }
    }
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

pub fn new_engine(board: Board, history: Vec<u64>) -> Engine<'static, GameHandler> {
    let options = EngineOptions::default();

    let shared = SearchSharedState {
        handler: GameHandler::default(),
        history: history,
        tt: TranspositionTable::default(),
    };

    Engine::new(board, options, shared)
}

#[test]
fn test_conversion() {
    use std::collections::HashMap;
    use engine::for_loop;
    use chess::{ALL_PIECES, ALL_SQUARES, ALL_COLORS};
    use engine::utils::positiongen::PositionGenerator;
    use engine::EvalType;
    use chess::BoardBuilder;

    let mut white_index_to_values = HashMap::new();
    let mut black_index_to_values = HashMap::new();
    for_loop!(piece ALL_PIECES, sq ALL_SQUARES, color ALL_COLORS; {
        let index = CurrentFeatures::white_feature_index(sq, piece, color);
        white_index_to_values.insert(index as u16, (sq, piece, color));

        let index = CurrentFeatures::black_feature_index(sq, piece, color);
        black_index_to_values.insert(index as u16, (sq, piece, color));
    });

    let boards = PositionGenerator::new().take(10000);

    for (i, board) in boards.enumerate() {
        let winner = match i % 3 {
            0 => None,
            1 => Some(Color::Black),
            2 => Some(Color::White),
            _ => unreachable!(),
        };
        let position = TrainingDataPosition {
            board,
            winner,
            eval: Eval::CentiPawn(i as EvalType),
        };
        let data_entry: TrainingDataEntry = position.into();

        let encoded = bincode::serialize(&data_entry).unwrap();
        assert_eq!(encoded.len(), ENTRY_SIZE_BYTES);
        // Example write to file
        let decoded: TrainingDataEntry = bincode::deserialize(&encoded).unwrap();

        let mut new_board = BoardBuilder::new();
        new_board.side_to_move(board.side_to_move());
        new_board.castle_rights(Color::White, board.castle_rights(Color::White));
        new_board.castle_rights(Color::Black, board.castle_rights(Color::Black));
        new_board.en_passant(board.en_passant().map(|s| s.get_file()));

        let index_to_values = match board.side_to_move() {
            Color::White => &white_index_to_values,
            Color::Black => &black_index_to_values,
        };

        for feature in decoded.our_features.iter() {
            if *feature == u16::MAX {
                break;
            }
            let (square, piece, color) = index_to_values.get(feature).expect("Invalid index");
            new_board.piece(*square, *piece, *color);
        }

        let new_board: Board = new_board.try_into().expect("Invalid board");

        assert_eq!(format!("{}", new_board), format!("{}", board));
    }
}

pub(crate) fn shared<T>(item: T) -> Arc<Mutex<T>> {
    Arc::new(Mutex::new(item))
}

#[macro_export]
macro_rules! create_io {
    ($name:expr, $type:ident, $($param:ident $value:expr),*) => {
        $type::new(
            OpenOptions::new()
            $(.$param($value))*
            .open($name)
            .unwrap()
        )
    };
}

#[macro_export]
macro_rules! clone {
    ($($var:ident)*) => {
        $(let $var = $var.clone();)*
    };
}
