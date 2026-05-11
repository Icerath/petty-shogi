use core::fmt;

use crate::{Piece, PieceKind, Side, Square};
pub trait BoardState {
    const EMPTY: Self;

    fn set_side_to(&mut self, to: Side) {
        _ = to;
    }

    fn set_piece_at(&mut self, piece: Piece, sq: Square, set: bool) {
        _ = piece;
        _ = sq;
        _ = set;
    }

    fn set_hand_size(&mut self, side: Side, piece: PieceKind, old: u8, new: u8) {
        _ = side;
        _ = piece;
        _ = old;
        _ = new;
    }

    fn debug(&self, debug_struct: &mut fmt::DebugStruct) -> fmt::Result {
        _ = debug_struct;
        Ok(())
    }
}

impl BoardState for () {
    const EMPTY: Self = ();
}
