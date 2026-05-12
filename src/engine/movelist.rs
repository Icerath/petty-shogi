use super::{Board, score::Score};
use crate::Move;

#[derive(Default)]
pub struct MoveList {
    index: usize,
    moves: Vec<(Move, Score)>,
    state: State,
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum State {
    #[default]
    Pv,
    HashMove {
        complete: bool,
    },
    GeneratedCaptures,
    Killer {
        complete: bool,
    },
    GeneratedNonCaptures,
}

impl MoveList {
    pub fn next(
        &mut self,
        board: &Board,
        captures_only: bool,
        tt_move: Option<Move>,
        killer: Option<Move>,
        pv_move: Option<Move>,
        order: impl Fn(Move) -> Score + Copy,
    ) -> Option<Move> {
        if let Some(mov) = self.next_move() {
            return Some(mov);
        }
        loop {
            if let Some(mov) = self.next_move() {
                return Some(mov);
            }
            match self.state {
                State::Pv => {
                    self.state = State::HashMove { complete: false };
                    if let Some(pv_move) = pv_move {
                        assert!(board.is_legal(pv_move));
                        break Some(pv_move);
                    }
                }
                State::HashMove { complete: false } => {
                    self.state = State::HashMove { complete: true };
                    if let Some(tt_move) = tt_move
                        && board.is_legal(tt_move)
                    {
                        break Some(tt_move);
                    }
                }
                State::HashMove { complete: true } => {
                    self.state = State::GeneratedCaptures;
                    self.generate_moves::<true>(board, tt_move, killer, order);
                }
                State::GeneratedCaptures if captures_only => break None,
                State::GeneratedCaptures => {
                    self.state = State::Killer { complete: false };
                }
                State::Killer { complete: false } => {
                    self.state = State::Killer { complete: true };
                    if let Some(killer) = killer
                        && board.is_legal(killer)
                    {
                        break Some(killer);
                    }
                }
                State::Killer { complete: true } => {
                    self.state = State::GeneratedNonCaptures;
                    self.generate_moves::<false>(board, tt_move, killer, order);
                }
                State::GeneratedNonCaptures => break None,
            }
        }
    }

    fn generate_moves<const CAPTURES: bool>(
        &mut self,
        board: &Board,
        tt_move: Option<Move>,
        killer: Option<Move>,
        order: impl Fn(Move) -> Score + Copy,
    ) {
        self.moves.clear();
        self.index = 0;
        let mask = if CAPTURES { board[!board.active] } else { !board[!board.active] };
        board.pseudolegal_moves_with(mask, |mov| self.moves.push((mov, order(mov))));
        if let Some(tt_move) = tt_move {
            self.remove_move(tt_move);
        }
        if let Some(killer) = killer {
            self.remove_move(killer);
        }
    }

    fn remove_move(&mut self, mov: Move) {
        let Some(index) = self.moves.iter().position(|(m, _)| *m == mov) else { return };
        self.moves.swap_remove(index);
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
        let mov = self.moves[best].0;
        self.moves.swap(self.index, best);
        self.index += 1;
        Some(mov)
    }
}
