use std::time::Instant;

use petty_shogi::Board;

fn main() {
    let start = Instant::now();
    let mut board = Board::start_pos();
    println!("total: {}", board.print_perft(2));
    println!("duration: {:?}", start.elapsed());
}
