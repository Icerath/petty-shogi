use super::score::Score;
use crate::{Board, Move};

pub struct MoveList {
    moves: Vec<(Move, Score)>,
    index: usize,
    state: State,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Start,
    GeneratedCaptures,
    GeneratedNonCaptures,
}

impl MoveList {
    pub fn new() -> Self {
        Self { moves: vec![], index: 0, state: State::Start }
    }

    pub fn next(
        &mut self,
        board: &Board,
        captures_only: bool,
        order: impl Fn(Move) -> Score + Copy,
    ) -> Option<Move> {
        match self.next_move() {
            Some(mov) => Some(mov),
            None => match self.state {
                State::Start => {
                    self.generate_moves::<true>(board, order);
                    self.state = State::GeneratedCaptures;
                    self.next(board, captures_only, order)
                }
                State::GeneratedCaptures if captures_only => None,
                State::GeneratedCaptures => {
                    self.generate_moves::<false>(board, order);
                    self.state = State::GeneratedNonCaptures;
                    self.next(board, captures_only, order)
                }
                State::GeneratedNonCaptures => None,
            },
        }
    }

    fn generate_moves<const CAPTURES: bool>(
        &mut self,
        board: &Board,
        order: impl Fn(Move) -> Score + Copy,
    ) {
        self.moves.clear();
        self.index = 0;
        let mask = if CAPTURES { board[!board.active] } else { !board[!board.active] };
        board.pseudolegal_moves_with(mask, |mov| self.moves.push((mov, order(mov))));
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
