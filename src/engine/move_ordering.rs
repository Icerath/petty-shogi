use super::{piece_value, score::Score};
use crate::{Board, Move};

pub fn order(board: &Board) -> impl Fn(Move) -> Score + Copy {
    move |mov: Move| {
        let mut score = 0;
        if let Move::Board { from, to, .. } = mov
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
        Score(score)
    }
}
