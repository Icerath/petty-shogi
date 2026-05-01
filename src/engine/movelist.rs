use crate::{Bitboard, Board, Move, movegen::FilterPromote};

pub struct MoveList {
    moves: Vec<Move>,
    index: usize,
    generated_noncaptures: bool,
}

impl MoveList {
    pub fn new(board: &Board) -> Self {
        let mut movelist = Self { moves: vec![], index: 0, generated_noncaptures: false };
        movelist.generate_moves(board[!board.active], board);
        movelist
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
                self.index = 0;
                self.generate_moves(!board[!board.active], board);
                self.next(board, captures_only)
            }
        }
    }

    fn generate_moves(&mut self, mask: Bitboard, board: &Board) {
        board.pseudolegal_moves_with(mask, FilterPromote::<true, _>(|mov| self.moves.push(mov)));
        board.pseudolegal_moves_with(mask, FilterPromote::<false, _>(|mov| self.moves.push(mov)));
    }
}
