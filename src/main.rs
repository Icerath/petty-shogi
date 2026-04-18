use std::time::Instant;

use action::Action;
use board::Board;
use square::Square;

mod action;
mod bitboard;
mod board;
mod movegen;
mod perft;
mod piece;
mod sfen;
mod side;
mod square;

fn main() {
    let start = Instant::now();
    let mut board = Board::start_pos();
    println!("{}", board.print_perft(5));
    println!("{:?}", start.elapsed());
}
