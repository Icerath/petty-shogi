use crate::{Board, Move};

pub struct MoveList {
    moves: Vec<Move>,
    index: usize,
    generated_noncaptures: bool,
}

impl MoveList {
    pub fn new(board: &Board) -> Self {
        Self {
            moves: board.pseudolegal_moves_with(board[!board.active], vec![]),
            index: 0,
            generated_noncaptures: false,
        }
    }
}

impl MoveList {
    pub fn next(&mut self, board: &Board, captures_only: bool) -> Option<Move> {
        match self.moves.get(self.index).copied() {
            Some(mov) => {
                self.index += 1;
                if board.is_legal(mov) { Some(mov) } else { self.next(board, captures_only) }
            }
            None if captures_only || self.generated_noncaptures => None,
            None => {
                self.generated_noncaptures = true;
                self.moves.clear();
                board.pseudolegal_moves_with(!board[!board.active], &mut self.moves);
                self.next(board, captures_only)
            }
        }
    }
}
