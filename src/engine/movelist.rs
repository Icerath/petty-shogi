use super::{piece_value, score::Score};
use crate::{Board, Move};

pub struct MoveList {
    moves: Vec<(Move, Score)>,
    index: usize,
    generated_noncaptures: bool,
}

impl MoveList {
    pub fn new(board: &Board, tt_move: Option<Move>) -> Self {
        let mut movelist = Self { moves: vec![], index: 0, generated_noncaptures: false };
        movelist.generate_moves::<true>(board, tt_move);
        movelist
    }

    pub fn next(
        &mut self,
        board: &Board,
        captures_only: bool,
        tt_move: Option<Move>,
    ) -> Option<Move> {
        match self.next_move() {
            Some(mov) => Some(mov),
            None if captures_only || self.generated_noncaptures => None,
            None => {
                self.generated_noncaptures = true;
                self.generate_moves::<false>(board, tt_move);
                self.next(board, captures_only, tt_move)
            }
        }
    }

    fn generate_moves<const CAPTURES: bool>(&mut self, board: &Board, tt_move: Option<Move>) {
        self.moves.clear();
        self.index = 0;
        let mask = if CAPTURES { board[!board.active] } else { !board[!board.active] };
        board.pseudolegal_moves_with(mask, |mov| {
            self.push_move::<CAPTURES>(board, mov, tt_move);
        });
    }

    fn push_move<const CAPTURES: bool>(&mut self, board: &Board, mov: Move, tt_move: Option<Move>) {
        let mut score = 0;
        if CAPTURES
            && let Move::Board { from, to, .. } = mov
            && board[!board.active].contains(to)
        {
            let from_piece = board.pieces.get(from).unwrap();
            let to_piece = board.pieces.get(to).unwrap();
            score += piece_value::board(to_piece) - piece_value::board(from_piece);
        }
        let mut board = board.clone();
        board.play(mov);
        if board.is_check() {
            score += 50;
        }
        if let Move::Board { promoted: true, .. } = mov {
            score += 50;
        }
        if tt_move == Some(mov) {
            score += 100;
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
