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
    println!("{}", Board::start_pos().perft(5));
    println!("{:?}", start.elapsed());
    Board::start_pos().legal_moves(&mut |m| println!("{m}"));
    println!("{}", Board::start_pos());
}
