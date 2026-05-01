use crate::{Bitboard, Board, Move};

pub struct MoveList {
    promotions: Vec<Move>,
    moves: Vec<Move>,
    index: usize,
    generated_noncaptures: bool,
}

impl MoveList {
    pub fn new(board: &Board) -> Self {
        let mut movelist =
            Self { promotions: vec![], moves: vec![], index: 0, generated_noncaptures: false };
        movelist.generate_moves(board[board.active], board);
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
                self.generate_moves(!board[!board.active], board);
                self.next(board, captures_only)
            }
        }
    }

    fn generate_moves(&mut self, mask: Bitboard, board: &Board) {
        board.pseudolegal_moves_with(mask, |mov| {
            if let Move::Board { promoted: true, .. } = mov {
                self.promotions.push(mov);
            } else {
                self.moves.push(mov);
            }
        });
        self.promotions.append(&mut self.moves);
        std::mem::swap(&mut self.moves, &mut self.promotions);
    }
}
