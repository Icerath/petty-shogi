use crate::{Piece, PieceKind, Side};

pub fn board(piece: Piece) -> i32 {
    let score = match (piece.kind(), piece.promoted()) {
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
    };
    if piece.side() == Side::Gote { -score } else { score }
}

pub fn hand(piece: Piece) -> i32 {
    let score = match piece.kind() {
        PieceKind::Pawn => 120,
        PieceKind::Lance => 320,
        PieceKind::Knight => 420,
        PieceKind::Silver => 520,
        PieceKind::Gold => 620,
        PieceKind::Bishop => 720,
        PieceKind::Rook => 820,
        PieceKind::King => 0,
    };
    if piece.side() == Side::Gote { -score } else { score }
}
