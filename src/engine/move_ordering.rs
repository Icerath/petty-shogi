use super::{
    Board,
    piece_value::{self, PSQT},
    score::Score,
};
use crate::Move;

pub fn order(board: &Board) -> impl Fn(Move) -> Score + Copy {
    move |mov: Move| {
        let mut score = 0;

        if let Move::Board { from, to, .. } = mov {
            let from_piece = board.pieces.get(from).unwrap();
            if let Some(capture) = board.pieces.get(to) {
                score += piece_value::board(capture) - piece_value::board(from_piece);
            } else {
                let piece = board.pieces.get(from).unwrap();
                score += PSQT[piece][to] - PSQT[piece][from];
            }
        }

        let mut board = board.clone();
        board.play(mov);
        if board.is_check() {
            score += 50;
        }
        if let Move::Board { promoted: true, .. } = mov {
            score += 50;
        }
        Score(score)
    }
}
