use super::{piece_value, score::Score};
use crate::{Bitboard, Board, Move};

pub struct MoveList {
    moves: Vec<(Move, Score)>,
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
        match self.next_move() {
            Some(mov) => Some(mov),
            None if captures_only || self.generated_noncaptures => None,
            None => {
                self.generated_noncaptures = true;
                self.generate_moves(!board[!board.active], board);
                self.next(board, captures_only)
            }
        }
    }

    fn generate_moves(&mut self, mask: Bitboard, board: &Board) {
        self.moves.clear();
        self.index = 0;
        board.pseudolegal_moves_with(mask, |mov| self.push_move(board, mov));
    }

    fn push_move(&mut self, board: &Board, mov: Move) {
        let mut score = 0;
        if let Move::Board { from, to, .. } = mov
            && board[!board.active].contains(to)
        {
            let from_piece = board.pieces.get(from).unwrap();
            let to_piece = board.pieces.get(to).unwrap();
            score += piece_value::board(to_piece) - piece_value::board(from_piece);
        }
        self.moves.push((mov, Score(score)));
    }

    fn next_move(&mut self) -> Option<Move> {
        if self.index >= self.moves.len() {
            return None;
        }
        let mut best = self.index;
        for index in (self.index + 1)..self.moves.len() {
            if self.moves[index].1 > self.moves[best].1 {
                best = index;
            }
        }
        self.moves.swap(self.index, best);
        let mov = self.moves[self.index].0;
        self.index += 1;
        Some(mov)
    }
}
