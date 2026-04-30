use super::*;
use crate::{Bitboard, Piece};

#[derive(Default, Clone)]
pub struct Search {
    pub nodes: u64,
}

impl<R: Fn(Response)> Engine<R> {
    pub fn search_root(&mut self, board: &Board, depth: u32, line: &mut Vec<Action>) -> Score {
        self.search(-Score::MAX, Score::MAX, board, depth, &mut NormalSearch { line })
    }

    fn search(
        &mut self,
        mut alpha: Score,
        beta: Score,
        board: &Board,
        depth: u32,
        kind: &mut impl SearchKind,
    ) -> Score {
        if depth == 0 {
            if kind.captures_only() {
                return self.shallow_eval(board);
            } else {
                return self.search(alpha, beta, board, u32::MAX, &mut CapturesOnly);
            }
        }
        assert!(alpha <= beta);
        if self.stop.is_stop() {
            return Score(0);
        }

        let mask = if kind.captures_only() { board[!board.active] } else { Bitboard::FULL };
        let pseudolegal_moves = board.pseudolegal_moves_with(mask, vec![]);

        let line_len = kind.line().map(|line| line.len()).unwrap_or(0);
        let mut best_line = vec![];
        let mut max_score = -Score::MAX;
        let mut no_moves = true;
        for mov in pseudolegal_moves {
            if !board.is_legal(mov) {
                continue;
            }
            no_moves = false;
            let mut board = board.clone();
            board.play(mov);

            let score = -self.search(-beta, -alpha, &board, depth - 1, kind).step();

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
            return if kind.captures_only() { self.shallow_eval(board) } else { -Score::MAX };
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
        let mut sum = 0;
        for piece in Piece::ALL {
            let mut mask = board[piece.kind()] & board[piece.side()];
            if piece.promoted() {
                mask &= board.pieces.promoted;
            }
            sum += mask.count() as i32 * piece_value::board(piece);
            sum += board.hands[piece.side()][piece.kind()] as i32 * piece_value::hand(piece);
        }
        sum
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
