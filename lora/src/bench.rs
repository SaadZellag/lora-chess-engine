use std::{
    time::{Duration, Instant},
    vec,
};

use engine::{SearchHandler, SearchResult, util::positiongen::PositionGenerator};
use std::rc::Rc;


const DEPTH: u8 = 4;

#[derive(Default)]
struct BenchHandler {
    nodes: u64,
    prev_result: Option<SearchResult>,
}

impl SearchHandler for BenchHandler {
    fn new_result(&mut self, result: SearchResult) {
        self.nodes += result.stats.nodes_visited;
        self.prev_result = Some(result);
    }

    fn should_stop(&self) -> bool {
        self.prev_result
            .map(|res| res.stats.depth >= DEPTH)
            .unwrap_or_default()
    }
}

pub fn bench() {
    let mut total_time = Duration::ZERO;
    let mut total_nodes = 0;

    let engine = engine::LoraEngine::new();
    let search_options = engine::SearchOptions {
        max_depth: DEPTH,
        ..engine::SearchOptions::new()
    };


    for board in PositionGenerator::new().take(200) {
        let mut handler = BenchHandler::default();
        let position = engine::SearchPosition {
            board,
            moves_played: Vec::new(),
        };

        let start = Instant::now();
        engine.search(position, search_options.clone(), &mut handler);

        total_time += start.elapsed();
        total_nodes += handler.nodes;
    }

    println!(
        "{} nodes {} nps",
        total_nodes,
        (total_nodes as f64 / total_time.as_secs_f64()) as u64,
    );
}
