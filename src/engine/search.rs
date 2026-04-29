use super::*;
use crate::Piece;

#[derive(Default, Clone)]
pub struct Search {
    pub nodes: u64,
}

impl<R: Fn(Response)> Engine<R> {
    pub fn search(
        &mut self,
        board: &Board,
        depth: u32,
        line: &mut Vec<Action>,
    ) -> ControlFlow<(), Score> {
        if depth == 0 {
            return ControlFlow::Continue(Score::CentiPawns(self.shallow_eval(board)));
        }
        if self.stop.is_stop() {
            return ControlFlow::Break(());
        }

        let outer_line = std::mem::take(line);
        let mut best = Score::CentiPawns(-i32::MAX);
        let mut best_line = vec![];
        board.pseudolegal_moves(|mov| {
            line.clear();

            let mut board = board.clone();
            board.play(mov);
            let score = self.search(&board, depth - 1, line)?.step();
            if self.stop.is_stop() {
                return ControlFlow::Break(());
            }
            if score > best {
                best = score;
                best_line = std::mem::take(line);
                best_line.push(mov);
            }
            ControlFlow::Continue(())
        })?;
        if best_line.is_empty() {
            // no legal moves
            return ControlFlow::Continue(Score::Mate(-1));
        }

        *line = outer_line;
        line.extend(best_line);
        ControlFlow::Continue(best)
    }

    fn shallow_eval(&mut self, board: &Board) -> i32 {
        self.search.nodes += 1;
        let absolute_score = self.abs_shallow_eval(board);
        match board.active {
            Side::Sente => absolute_score,
            Side::Gote => -absolute_score,
        }
    }

    fn abs_shallow_eval(&self, board: &Board) -> i32 {
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
