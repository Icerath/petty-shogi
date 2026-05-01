use std::fmt;

use crate::{Piece, PieceKind, Side, Square};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Zobrist(pub u64);

impl Zobrist {
    /// The zobrist for an empty board
    pub const EMPTY: Self = Self(0);

    pub fn xor_side_to_move(&mut self) {
        self.0 ^= TABLE.side_to_move;
    }

    pub fn xor_board_piece(&mut self, sq: Square, piece: Piece) {
        self.0 ^= TABLE.board_pieces[sq][piece];
    }

    pub fn xor_hand_piece(&mut self, side: Side, kind: PieceKind, count: u8) {
        self.0 ^= TABLE.hand_pieces[side][kind][count as usize];
    }
}

impl fmt::Debug for Zobrist {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

struct Rng(u64);

impl Rng {
    #[expect(clippy::cast_possible_truncation)]
    const fn next(&mut self) -> u64 {
        // copied from https://github.com/smol-rs/fastrand
        // Constants for WyRand taken from: https://github.com/wangyi-fudan/wyhash/blob/master/wyhash.h#L151
        const WY_CONST_0: u64 = 0x2d35_8dcc_aa6c_78a5;
        const WY_CONST_1: u64 = 0x8bb8_4b93_962e_acc9;

        let s = self.0.wrapping_add(WY_CONST_0);
        self.0 = s;
        let t = s as u128 * (s ^ WY_CONST_1) as u128;
        t as u64 ^ (t >> 64) as u64
    }
}

static TABLE: Table = Table::generate(Rng(0));

struct Table {
    side_to_move: u64,
    board_pieces: [[u64; Piece::LEN]; Square::LEN],
    hand_pieces: [[[u64; 18]; PieceKind::LEN]; Side::LEN],
}

impl Table {
    const fn generate(mut rng: Rng) -> Self {
        Self {
            side_to_move: rng.next(),
            board_pieces: konst::array::from_fn!(|_| konst::array::from_fn!(|_| rng.next())),
            hand_pieces: konst::array::from_fn!(|_| konst::array::from_fn!(
                |_| konst::array::from_fn!(|_| rng.next())
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::Zobrist;
    use crate::Board;

    #[test]
    fn test_zobrist_basics() {
        let board = Board::start_pos();
        let mut zobrists: HashMap<Zobrist, Board> = HashMap::new();
        _ = board.try_perft(5, &mut |board| {
            if let Some(previous) = zobrists.insert(board.zobrist, board.clone()) {
                assert!(
                    board.pieces == previous.pieces
                        && board.active == previous.active
                        && board.hands == previous.hands
                );
            }
            false
        });
    }
}
