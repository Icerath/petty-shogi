use super::{Engine, Score, piece_value};
use crate::{
    Action, Board, Piece, PieceKind, Side,
    engine::{movelist::MoveList, transposition_table::Nodetype},
};

#[derive(Default, Clone)]
pub struct Search {
    pub nodes: u64,
    pub depth_from_root: u64,
}

impl Engine {
    pub fn search_root(&mut self, board: &Board, depth: u32, line: &mut Vec<Action>) -> Score {
        self.search(-Score::MAX, Score::MAX, board, depth, &mut NormalSearch { line })
    }

    fn search<K: SearchKind>(
        &mut self,
        mut alpha: Score,
        beta: Score,
        board: &Board,
        depth: u32,
        kind: &mut K,
    ) -> Score {
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
        if !kind.captures_only()
            && self.search.depth_from_root > 0
            && let Some(entry) = self.ttable.get(board.zobrist, alpha, beta, depth)
        {
            return entry.score;
        }

        let mut movelist = MoveList::new(board);

        let line_len = kind.line().map_or(0, |line| line.len());
        let mut best_line = vec![];
        let mut max_score = -Score::MAX;
        let mut no_moves = true;
        while let Some(mov) = movelist.next(board, kind.captures_only()) {
            no_moves = false;
            let mut board = board.clone();
            board.play(mov);

            self.search.depth_from_root += 1;
            let score = -self.search(-beta, -alpha, &board, depth - 1, kind).step();
            self.search.depth_from_root -= 1;

            if self.stop.is_stop() {
                return score;
            }

            if let Some(line) = kind.line() {
                if score > max_score {
                    best_line.clear();
                    best_line.push(mov);
                    best_line.extend(line.iter().copied());
                }
                line.truncate(line_len);
            }

            max_score = score.max(max_score);
            alpha = alpha.max(max_score);

            if alpha >= beta {
                break;
            }
        }

        if no_moves {
            return if kind.captures_only() { self.shallow_eval(board) } else { -Score::MATE };
        }

        if !kind.captures_only() {
            let nodetype = if alpha >= beta {
                Nodetype::Beta
            } else if alpha == max_score {
                Nodetype::Exact
            } else {
                Nodetype::Alpha
            };
            self.ttable.insert(board.zobrist, depth, max_score, nodetype);
        }

        if let Some(parent_line) = kind.line() {
            parent_line.extend(best_line);
        }
        max_score
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
            sum_both += if side == Side::Sente { sum } else { -sum };
        }
        sum_both
    }
}

struct NormalSearch<'a> {
    line: &'a mut Vec<Action>,
}

struct CapturesOnly;

trait SearchKind {
    fn line(&mut self) -> Option<&mut Vec<Action>>;
    fn captures_only(&self) -> bool;
}

impl SearchKind for NormalSearch<'_> {
    fn line(&mut self) -> Option<&mut Vec<Action>> {
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

    fn line(&mut self) -> Option<&mut Vec<Action>> {
        None
    }
}
