use std::time::Instant;

use board::Board;

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
