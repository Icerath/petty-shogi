use super::{Board, Engine, Score};
use crate::{
    Bitboard, Move, Side,
    engine::{movelist::MoveList, transposition_table::Nodetype},
};

#[derive(Default, Clone)]
pub struct Search {
    pub nodes: u64,
    pub depth_from_root: u32,
    pub max_seldepth: u32,
    pub killer: Vec<Option<Move>>,
    pub fail_high: u64,
    pub fail_high_test: u64,
    pub is_pv: bool,
    pub prev_pv: Vec<Move>,
    pub current_line: Vec<Move>,
}

impl Engine {
    pub fn search_root(&mut self, board: &mut Board, depth: u32, line: &mut Vec<Move>) -> Score {
        self.search.killer.clear();
        self.search.killer.extend([None; 64]);
        self.search.is_pv = true;

        self.search(-Score::MAX, Score::MAX, board, depth, &mut NormalSearch { line })
    }

    #[expect(clippy::too_many_lines)]
    fn search<K: SearchKind>(
        &mut self,
        mut alpha: Score,
        beta: Score,
        board: &mut Board,
        depth: u32,
        kind: &mut K,
    ) -> Score {
        self.search.max_seldepth = self.search.max_seldepth.max(self.search.depth_from_root);
        debug_assert!(alpha <= beta);
        let alpha_orig = alpha;

        if depth == 0 {
            if kind.captures_only() {
                return self.shallow_eval(board);
            }
            return self.search(alpha, beta, board, u32::MAX, &mut CapturesOnly);
        }

        let mut tt_move = None;
        if !kind.captures_only()
            && self.search.depth_from_root > 0
            && let Some(entry) = self.ttable.get(board.state.zobrist)
        {
            if let Some(score) = entry.score(alpha, beta, depth) {
                if let (Some(mov), Some(line)) = (entry.mov, kind.line()) {
                    line.push(mov);
                }
                return score;
            }
            tt_move = entry.mov;
        }

        let is_pv = self.search.is_pv;
        let pv_move = if !kind.captures_only()
            && is_pv
            && let Some(mov) = self.search.prev_pv.get(self.search.depth_from_root as usize)
            && self.search.prev_pv[..self.search.depth_from_root as usize]
                == self.search.current_line
        {
            Some(*mov)
        } else {
            None
        };

        let line_len = kind.line().map_or(0, |line| line.len());

        // null move pruning
        if !kind.captures_only() && depth > 3 && !board.is_check() && pv_move.is_none() {
            board.switch_side();
            self.search.depth_from_root += 1;
            let score = -self.search(-beta, -alpha, board, depth / 3, kind);
            self.search.depth_from_root -= 1;
            board.switch_side();

            if let Some(line) = kind.line() {
                line.truncate(line_len);
            }
            if score >= beta {
                return beta;
            }
        }

        let mut best_score =
            if kind.captures_only() { self.shallow_eval(board) } else { -Score::MATE };
        let mut best_move = None;

        if kind.captures_only() && !board.is_check() {
            if best_score >= beta {
                return best_score;
            } else if best_score > alpha {
                alpha = best_score;
            }
        }

        let mut move_count = 0;
        let mut best_line = vec![];
        let mut movelist = MoveList::default();
        let mut killer_move = (!kind.captures_only())
            .then(|| self.search.killer[self.search.depth_from_root as usize])
            .flatten();

        if killer_move.is_some() && killer_move == pv_move || killer_move == tt_move {
            killer_move = None;
        }
        if tt_move.is_some() && tt_move == pv_move {
            tt_move = None;
        }
        while let Some(mov) = movelist.next(
            board,
            kind.captures_only(),
            tt_move,
            killer_move,
            pv_move,
            super::move_ordering::order(board),
        ) {
            if self.stop.is_stop() {
                return Score(0);
            }
            let mut board = board.clone();
            board.play(mov);
            if !board.was_legal(mov) {
                continue;
            }
            move_count += 1;

            let mut next_depth = depth - 1;

            let late_move_reduction = depth > 2 && move_count >= 2;

            if !kind.captures_only() {
                if board.is_check() {
                    next_depth += 1;
                }
                if late_move_reduction {
                    next_depth -= 1;
                }
            }

            self.search.depth_from_root += 1;
            self.search.current_line.push(mov);
            let mut score = -self.search(-beta, -alpha, &mut board, next_depth, kind).step();
            // repeat search if late move reduction search fails high
            if !kind.captures_only() && score >= beta && late_move_reduction {
                if let Some(line) = kind.line() {
                    line.truncate(line_len);
                }
                score = -self.search(-beta, -alpha, &mut board, next_depth + 1, kind).step();
            }
            self.search.current_line.pop();
            self.search.depth_from_root -= 1;

            if let Some(line) = kind.line() {
                if score > best_score {
                    best_line.clear();
                    best_line.push(mov);
                    best_line.extend(line[line_len..].iter().copied());
                }
                line.truncate(line_len);
            }

            if score > best_score {
                best_score = score;
                best_move = Some(mov);
            }

            alpha = alpha.max(best_score);

            self.search.fail_high_test += 1;
            if alpha >= beta {
                self.search.fail_high += 1;
                if !kind.captures_only() && !board.pieces.contains(mov.to()) {
                    self.search.killer[self.search.depth_from_root as usize] = Some(mov);
                }
                break;
            }
            self.search.is_pv = false;
        }
        self.search.is_pv = is_pv;

        if move_count == 0 {
            return best_score;
        }

        if !kind.captures_only() {
            let nodetype = if best_score <= alpha_orig {
                Nodetype::Alpha
            } else if best_score >= beta {
                Nodetype::Beta
            } else {
                Nodetype::Exact
            };
            self.ttable.insert(board.state.zobrist, depth, best_score, best_move, nodetype);
        }

        if let Some(parent_line) = kind.line() {
            parent_line.extend(best_line);
        }
        best_score
    }

    fn shallow_eval(&mut self, board: &Board) -> Score {
        let absolute_score = self.abs_shallow_eval(board);
        Score(match board.active {
            Side::Sente => absolute_score,
            Side::Gote => -absolute_score,
        })
    }

    fn abs_shallow_eval(&mut self, board: &Board) -> i32 {
        self.search.nodes += 1;
        let mut sum_both = 0;
        for side in Side::ALL {
            let mut sum = 0;
            {
                let protected_squares = {
                    let mut board = board.without_state().clone();
                    board.active = side;
                    board.pseudolegal_moves_all(Bitboard::FULL, Bitboard::EMPTY)
                };
                sum -= i32::from((board[side] & !protected_squares).count()) * 10;
                sum -= i32::from(
                    ((!side).promotion_zone() & !protected_squares & !board[side]).count(),
                ) * 10;
            }
            sum_both += if side == Side::Sente { sum } else { -sum };
        }
        let initiative = match board.active {
            Side::Sente => 5,
            Side::Gote => -5,
        };
        board.state.piece_values.0 + sum_both + initiative
    }
}

struct NormalSearch<'a> {
    line: &'a mut Vec<Move>,
}

struct CapturesOnly;

trait SearchKind {
    fn line(&mut self) -> Option<&mut Vec<Move>>;
    fn captures_only(&self) -> bool;
}

impl SearchKind for NormalSearch<'_> {
    fn line(&mut self) -> Option<&mut Vec<Move>> {
        Some(self.line)
    }

    fn captures_only(&self) -> bool {
        false
    }
}

impl SearchKind for CapturesOnly {
    fn captures_only(&self) -> bool {
        true
    }

    fn line(&mut self) -> Option<&mut Vec<Move>> {
        None
    }
}
