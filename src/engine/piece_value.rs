use crate::{Piece, PieceKind};

#[expect(clippy::match_same_arms)]
pub fn board(piece: Piece) -> i32 {
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
