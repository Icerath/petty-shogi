use super::{Engine, Score, piece_value};
use crate::{
    Board, Move, Piece, PieceKind, Side,
    engine::{movelist::MoveList, transposition_table::Nodetype},
};

#[derive(Default, Clone)]
pub struct Search {
    pub nodes: u64,
    pub depth_from_root: u32,
    pub max_seldepth: u32,
}

impl Engine {
    pub fn search_root(&mut self, board: &mut Board, depth: u32, line: &mut Vec<Move>) -> Score {
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
        if depth == 0 {
            if kind.captures_only() {
                return self.shallow_eval(board);
            }
            return self.search(alpha, beta, board, u32::MAX, &mut CapturesOnly);
        }
        assert!(alpha <= beta);
        if self.stop.is_stop() {
            return Score(0);
        }

        let mut tt_move = None;

        if !kind.captures_only()
            && self.search.depth_from_root > 0
            && let Some(entry) = self.ttable.get(board.zobrist)
        {
            if let Some(score) = entry.score(alpha, beta, depth) {
                if let (Some(mov), Some(line)) = (entry.mov, kind.line()) {
                    line.push(mov);
                }
                return score;
            }
            tt_move = entry.mov;
        }

        let line_len = kind.line().map_or(0, |line| line.len());

        // null move pruning
        if !kind.captures_only() && depth > 3 && !board.is_check() {
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
        let mut movelist = MoveList::new();
        while let Some(mov) =
            movelist.next(board, kind.captures_only(), super::move_ordering::order(board, tt_move))
        {
            let mut board = board.clone();
            board.play(mov);
            if !board.was_legal(mov) {
                continue;
            }
            move_count += 1;

            let mut next_depth = depth - 1;

            let late_move_reduction = || depth > 2 && move_count >= 2;

            if !kind.captures_only() {
                if board.is_check() {
                    next_depth += 1;
                }
                if late_move_reduction() {
                    next_depth -= 1;
                }
            }

            self.search.depth_from_root += 1;
            let mut score = -self.search(-beta, -alpha, &mut board, next_depth, kind).step();
            self.search.depth_from_root -= 1;

            if self.stop.is_stop() {
                return score;
            }

            if !kind.captures_only() && score >= beta && late_move_reduction() {
                // repeat search if late move reduction search fails high
                score = -self.search(-beta, -alpha, &mut board, next_depth + 1, kind).step();
            }

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

            if alpha >= beta {
                break;
            }
        }

        if move_count == 0 {
            return best_score;
        }

        if !kind.captures_only() {
            let nodetype = if alpha >= beta {
                Nodetype::Beta
            } else if alpha == best_score {
                Nodetype::Exact
            } else {
                Nodetype::Alpha
            };
            self.ttable.insert(board.zobrist, depth, best_score, best_move, nodetype);
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
            for promoted in [false, true] {
                for kind in PieceKind::ALL {
                    let mut mask = board[kind] & board[side];
                    if promoted {
                        mask &= board.pieces.promoted;
                    }
                    sum += i32::from(mask.count())
                        * piece_value::board(Piece::new(side, kind, promoted));
                    sum += i32::from(board.hands[side][kind]) * piece_value::hand(kind);
                }
            }
            let mut board = board.clone();
            board.active = side;
            sum += board.pseudolegal_moves_all(board[side], 0i32) * 5;
            sum_both += if side == Side::Sente { sum } else { -sum };
        }
        sum_both
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
