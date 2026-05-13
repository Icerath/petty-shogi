use crate::{Piece, PieceKind, Side, Square, board_state::BoardState};

#[expect(clippy::match_same_arms)]
pub const fn board(piece: Piece) -> i32 {
    match (piece.kind(), piece.promoted()) {
        (PieceKind::Pawn, false) => 100,
        (PieceKind::Pawn, true) => 600,
        (PieceKind::Lance, false) => 200,
        (PieceKind::Lance, true) => 550,
        (PieceKind::Knight, false) => 300,
        (PieceKind::Knight, true) => 500,
        (PieceKind::Silver, false) => 400,
        (PieceKind::Silver, true) => 500,
        (PieceKind::Gold, _) => 500,
        (PieceKind::Bishop, _) => 700,
        (PieceKind::Rook, _) => 800,
        (PieceKind::King, _) => 0,
    }
}

pub fn hand(piece: PieceKind) -> i32 {
    match piece {
        PieceKind::Pawn => 120,
        PieceKind::Lance => 320,
        PieceKind::Knight => 420,
        PieceKind::Silver => 520,
        PieceKind::Gold => 620,
        PieceKind::Bishop => 720,
        PieceKind::Rook => 820,
        PieceKind::King => 0,
    }
}

#[expect(clippy::cast_possible_truncation)]
pub static PSQT: [[i32; Square::LEN]; Piece::LEN] =
    konst::array::from_fn!(|i| psqt(Piece::from_int(i as u8).unwrap()));

const fn psqt(piece: Piece) -> [i32; Square::LEN] {
    #[rustfmt::skip]
    let table = match (piece.kind(), piece.promoted()) {
        (PieceKind::King, _) => [
             0,  0,  0,  0,  0,  0,  0,  0,  0,
             0,  0,  0,  0,  0,  0,  0,  0,  0,
             0,  0,  0,  0,  0,  0,  0,  0,  0,
             0,  0,  0,  0,  0,  0,  0,  0,  0,
             0,  0,  0,  0,  0,  0,  0,  0,  0,
             0,  0,  0,  0,  0,  0,  0,  0,  0,
             0, 20,  5,  5,  0,  5,  5, 20,  0,
            30, 50, 30, 10,  0, 10, 30, 50, 30,
            30, 50, 30, 10,  0, 10, 30, 50, 30,
        ],
        (PieceKind::Pawn, false) => [
             0,  0,  0,  0,  0,  0,  0,  0,  0,
             0,  0,  0,  0,  0,  0,  0,  0,  0,
             0,  0,  0,  0,  0,  0,  0,  0,  0,
            30, 30, 30, 30, 30, 30, 30, 30, 30,
            10, 10, 10, 20, 20, 20, 10, 10, 10,
            10, 10, 10, 10, 10, 10, 10, 10, 10,
            10, 10, 10,  0,  0,  0, 10, 10, 10,
             0,  0,  0,  0,  0,  0,  0,  0,  0,
             0,  0,  0,  0,  0,  0,  0,  0,  0,
        ],
        _ => [0; 81],
    };
    let table = konst::array::from_fn!(|i| table[i] + board(piece));

    match piece.side() {
        Side::Sente => table,
        Side::Gote => konst::array::from_fn!(|i| -table[80 - i]),
    }
}

#[derive(Clone, Copy)]
pub struct PieceValues(pub i32);

impl BoardState for PieceValues {
    const EMPTY: Self = Self(0);

    fn set_hand_size(&mut self, side: Side, piece: PieceKind, old: u8, new: u8) {
        let diff = i32::from(new) - i32::from(old);
        let score = hand(piece) * diff;
        match side {
            Side::Sente => self.0 += score,
            Side::Gote => self.0 -= score,
        }
    }

    fn set_piece_at(&mut self, piece: Piece, sq: Square, set: bool) {
        if set {
            self.0 += PSQT[piece][sq];
        } else {
            self.0 -= PSQT[piece][sq];
        }
    }
}
