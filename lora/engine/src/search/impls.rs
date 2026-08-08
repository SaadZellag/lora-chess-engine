use chess::{ChessMove, Piece};

use crate::{EngineOptions, Eval, MAX_DEPTH, SearchHandler, SearchResult, SearchStats, search::{SearchOptions, SearchPosition, position::Position, tt::{EntryType, TTEntry, TranspositionTable}}};
use std::rc::Rc;

impl SearchPosition {
    pub fn new() -> Self {
        Self {
            board: chess::Board::default(),
            moves_played: Vec::new(),
        }
    }
}

impl SearchOptions {
    pub fn new() -> Self {
        Self {
            max_depth: u8::MAX,
            max_nodes: u64::MAX,
            mate_search_depth: None,
            moves_to_search: None,
        }
    }
}


impl<'a, H: SearchHandler> super::EngineSearcher<'a, H> {
    pub fn new(position: SearchPosition, engine_options: EngineOptions, search_options: super::SearchOptions, handler: &'a mut H) -> Self {
        Self {
            search_state: super::SearchState {
                nodes_searched: 0,
                depth_reached: 0,
                selective_depth_reached: 0,
                history_hash: Vec::new(),
                table_hits: 0,
                transposition_table: TranspositionTable::new(engine_options.tt_size_bytes),
            },
            engine_options,
            search_options,
            handler,
            position,
        }
    }

    pub fn best_move(&mut self) -> Option<SearchResult> {
        
        let mut startpos = Position::new(self.position.board.clone());
        self.search_state.history_hash.push(startpos.board().get_hash());

        for mv in self.position.moves_played.iter() {
            startpos = startpos.make_move(*mv);
            self.search_state.history_hash.push(startpos.board().get_hash());
        }
        


        // TODO: Checking if only 1 move possible, then playing that

        // Running depth 1 without quiese at least have a move
        // TODO: Remove this when search explosion problem is fixed
        let mut quiese = true;

        let mut res: Option<SearchResult> = None;

        for depth in 1..=self.search_options.max_depth {
            if let Some(most_recent) =
                self.search(startpos, depth, quiese, res.map(|r| r.best_move))
            {
                quiese = true;
                self.handler.new_result(most_recent);
                res = Some(most_recent);

                // Preventing from looking further if mate is forced
                match most_recent.eval {
                    Eval::MateIn(_) | Eval::MatedIn(_) => break,
                    _ => {}
                }
            } else {
                break;
            }
        }

        res
    }

    fn search(
        &mut self, 
        startpos: Position, 
        depth: u8, 
        quiese: bool, 
        prev_best_move: Option<chess::ChessMove>
    ) -> Option<SearchResult> {
        let mut best_mv = prev_best_move;
        self.search_state.depth_reached = depth;
        self.search_state.selective_depth_reached = depth;

        let mut best_eval = if let Some(best_mv) = best_mv {
            let copy = startpos.make_move(best_mv);
            -self.search_inner(
                &copy,
                depth - 1,
                Eval::WORST_EVAL,
                Eval::BEST_EVAL,
                quiese,
            )?
        } else {
            Eval::MIN
        };

        for mv in startpos.possible_moves() {
            // If the current move explored was the last best move, skip it since it was already searched
            if best_mv.unwrap_or_default() == mv {
                continue;
            }

            let copy = startpos.make_move(mv);
            let eval = -self.search_inner(
                &copy,
                depth - 1,
                Eval::WORST_EVAL,
                -best_eval,
                quiese,
            )?;

            // println!("{} {:?} {:?}", mv, eval, best_eval);

            if eval > best_eval {
                best_eval = eval;
                best_mv = Some(mv);
            }
        }

        let mut moves = [ChessMove::default(); MAX_DEPTH as usize];
        let mut current_mv = best_mv.expect("No moves found");
        let mut current_board = startpos;

        for i in 0..self.search_state.depth_reached {
            moves[i as usize] = current_mv;
            current_board = current_board.make_move(current_mv);
            if let Some(ttentry) = self.search_state.transposition_table.get(&current_board) {
                // assert_eq!(ttentry.flag, EntryType::Exact);
                current_mv = ttentry.mv;
                if !current_board.board().legal(current_mv) {
                    panic!(
                        "{} received move {} from tt entry {:?} | Board hash {}",
                        current_board.board(),
                        current_mv,
                        ttentry,
                        current_board.board().get_hash()
                    );
                }

                if ttentry.flag != EntryType::Exact {
                    break;
                }
            } else {
                break;
            }
        }

        best_mv.map(|mv| SearchResult {
            best_move: mv,
            eval: best_eval,
            stats: SearchStats {
                depth: self.search_state.depth_reached,
                sel_depth: self.search_state.selective_depth_reached,
                nodes_visited: self.search_state.nodes_searched,
                tbl_hits: self.search_state.table_hits,
            },
            hashfull: self.search_state.transposition_table.hashfull(),
            pv: moves,
        })
    }

    fn search_inner(
        &mut self,
        pos: &Position,
        mut depth: u8,
        mut alpha: Eval,
        mut beta: Eval,
        quiese: bool,
    ) -> Option<Eval> {
        if self.handler.should_stop() {
            return None;
        }

        let orig_alpha = alpha;
        let board = pos.board();

        // Checking for transposition
        let ttentry = self.search_state.transposition_table.get(pos);
        if let Some(ttentry) = ttentry {
            if ttentry.depth >= depth {
                self.search_state.table_hits += 1;
                match ttentry.flag {
                    EntryType::Exact => return Some(ttentry.eval),
                    EntryType::LowerBound => alpha = alpha.max(ttentry.eval),
                    EntryType::UpperBound => beta = beta.min(ttentry.eval),
                    EntryType::Invalid => {
                        panic!("Invalid entry") // Just in case
                    }
                }
                if alpha >= beta {
                    return Some(ttentry.eval);
                }
            }
        }

        let hash = board.get_hash();
        self.search_state.history_hash.push(hash);

        // Check extension
        // https://www.chessprogramming.org/Check_Extensions
        let in_check = board.checkers().popcnt() != 0;
        if in_check {
            depth += 1;
        }

        macro_rules! _return {
            ($eval:expr) => {
                self.search_state.history_hash.pop();
                return $eval;
            };
        }

        // Check for repetition
        if self.repetitions(hash) > 0 {
            _return!(Some(Eval::NEUTRAL));
        }

        self.search_state.nodes_searched += 1;

        if depth == 0 {
            _return!(if quiese {
                self.quiese(pos, 1, alpha, beta)
            } else {
                Some(pos.eval())
            });
        }

        let mut itt = pos.possible_moves();
        if itt.len() == 0 {
            _return!(if quiese {
                self.quiese(pos, 1, alpha, beta)
            } else {
                match board.checkers().popcnt() {
                    0 => Some(Eval::NEUTRAL),
                    _ => Some(Eval::MatedIn(pos.ply())),
                }
            });
        }

        // Null move pruning
        const R: u8 = 2;

        // Checking whether NMP is applicable
        let our_pieces = board.color_combined(board.side_to_move());
        let pawns = board.pieces(Piece::Pawn);
        let our_pawns = our_pieces & pawns;
        let only_pawns = our_pawns.popcnt() == our_pieces.popcnt() - 1; // the king is always there

        let do_nmp = our_pieces.popcnt() >= 8 && !only_pawns; // The 8 is just me who picked it

        if do_nmp {
            if let Some(new_board) = pos.null_move() {
                let score = -self.search_inner(
                    &new_board,
                    depth.saturating_sub(R + 1),
                    -beta,
                    -beta + Eval::UNIT,
                    quiese,
                )?;

                if score >= beta {
                    _return!(Some(score));
                }
            }
        }

        let mut b_search_pv = true;

        let mut score = Eval::MIN;
        let mut best_mv = ChessMove::default();

        if let Some(ttentry) = ttentry {
            let mv = ttentry.mv;
            assert!(
                board.legal(mv),
                "Got an invalid move {} for position {} from the TT",
                mv,
                board
            );

            itt.remove_move(mv);

            best_mv = mv;
            let copy = pos.make_move(mv);
            score = -self.search_inner(&copy, depth - 1, -beta, -alpha, quiese)?;

            if score > alpha {
                alpha = score;
                b_search_pv = false;
            }

            if alpha >= beta {
                itt.allow_only(chess::EMPTY);
            }
        }

        for mv in itt {
            let copy = pos.make_move(mv);

            let mut current_score;

            if b_search_pv {
                current_score =
                    -self.search_inner(&copy, depth - 1, -beta, -alpha, quiese)?;
            } else {
                current_score = -self.search_inner(
                    &copy,
                    depth - 1,
                    -alpha - Eval::UNIT,
                    -alpha,
                    quiese,
                )?;
                if current_score > alpha && current_score < beta {
                    current_score = -self.search_inner(
                        &copy,
                        depth - 1,
                        -beta,
                        -alpha,
                        quiese,
                    )?;
                }
            };

            if current_score > score {
                score = current_score;
                best_mv = mv;
            }

            if score > alpha {
                alpha = score;
                b_search_pv = false;
            }

            if alpha >= beta {
                break;
            }
        }

        // if best_mv == Eval::MIN {
        //     panic!("Score has not been updated");
        // }
        // if pos.ply() < DEBUG_DEPTH {
        //     println!("{}Final Eval: {:?}", tabs, best_score);
        // }

        // Using failsoft
        // let mut best_score = {
        //     let copy = pos.make_move(itt.next().unwrap(), self.nnue);
        //     -self.search_node(&copy, depth - 1, stats, shared, -beta, -alpha, quiese)?
        // };

        // if best_score > alpha {
        //     if best_score >= beta {
        //         _return!(Some(best_score));
        //     }
        //     alpha = best_score;
        // }

        // for mv in itt {
        //     let copy = pos.make_move(mv, self.nnue);
        //     let mut score = -self.search_node(
        //         &copy,
        //         depth - 1,
        //         stats,
        //         shared,
        //         -alpha - Eval::UNIT,
        //         -alpha,
        //         quiese,
        //     )?;

        //     if score > alpha && score < beta {
        //         // Aspiration window failed, costly research must be done
        //         score =
        //             -self.search_node(&copy, depth - 1, stats, shared, -beta, -alpha, quiese)?;
        //         alpha = alpha.max(score);
        //     }

        //     if score > best_score {
        //         if score >= beta {
        //             _return!(Some(score));
        //         }
        //         best_score = score;
        //     }
        // }

        // Storing in tt table
        let flag = if score <= orig_alpha {
            EntryType::UpperBound
        } else if score >= beta {
            EntryType::LowerBound
        } else {
            EntryType::Exact
        };

        let ttentry = TTEntry {
            hash: board.get_hash(),
            flag,
            depth: depth,
            eval: score,
            mv: best_mv,
        };

        self.search_state.transposition_table.set(pos, ttentry);

        _return!(Some(score));
    }

    fn repetitions(&self, hash: u64) -> usize {
        self.search_state.history_hash
            .iter()
            .rev()
            .step_by(2)
            .skip(1)
            .filter(|&&h| h == hash)
            .count()
    }
}