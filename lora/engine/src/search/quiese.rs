use crate::SearchHandler;
use crate::search::position::Position;
use crate::{Eval, SearchStats};

impl<'a, H: SearchHandler> super::EngineSearcher<'a, H> {
    // https://www.chessprogramming.org/Quiescence_Search
    pub(crate) fn quiese(
        &mut self,
        pos: &Position,
        current_depth: u8,
        mut alpha: Eval,
        beta: Eval,
    ) -> Option<Eval> {
        if self.handler.should_stop() {
            return None;
        }

        let itt = pos.possible_captures();

        let current_score = pos.eval();
        self.search_state.nodes_searched += 1;
        
        if self.search_state.nodes_searched >= self.search_options.max_nodes {
            return None;
        }

        if itt.len() == 0 {
            return Some(current_score);
        }

        self.search_state.selective_depth_reached = self.search_state.selective_depth_reached.max(current_depth + self.search_state.depth_reached);

        if current_score >= beta {
            return Some(beta);
        }
        if current_score > alpha {
            alpha = current_score;
        }

        for mv in itt {
            let copy = pos.make_move(mv);
            let eval = -self.quiese(&copy, current_depth + 1, -beta, -alpha)?;

            if eval >= beta {
                return Some(beta);
            }

            if eval > alpha {
                alpha = eval;
            }
        }

        Some(alpha)
    }
}
