use crate::{Bitboard, Side, Square};

pub fn rook_moves(sq: Square, occupancy: Bitboard) -> Bitboard {
    ROOK_TABLE[ROOK_ENTRIES[sq].index(occupancy)]
}

pub fn bishop_moves(sq: Square, occupancy: Bitboard) -> Bitboard {
    BISHOP_TABLE[BISHOP_ENTRIES[sq].index(occupancy)]
}

pub fn lance_moves(sq: Square, occupancy: Bitboard, side: Side) -> Bitboard {
    ROOK_TABLE[ROOK_ENTRIES[sq].index(occupancy)] & LANCE_MASKS[side][sq]
}

include!("./magic_entries.rs");

const ROOK_TABLE_DATA: ([u8; 8388608], u128) = (*include_bytes!("./rook_table.bin"), 0);
const BISHOP_TABLE_DATA: ([u8; 323584], u128) = (*include_bytes!("./bishop_table.bin"), 0);

static ROOK_TABLE: &[Bitboard] = unsafe { cast_from_bytes(&ROOK_TABLE_DATA.0) };
static BISHOP_TABLE: &[Bitboard] = unsafe { cast_from_bytes(&BISHOP_TABLE_DATA.0) };

const unsafe fn cast_from_bytes<T>(slice: &[u8]) -> &[T] {
    unsafe { std::slice::from_raw_parts(slice.as_ptr().cast(), slice.len() / size_of::<T>()) }
}

const LANCE_MASKS: [[Bitboard; Square::LEN]; 2] = [
    konst::array::from_fn!(|i| lance_mask(Side::Sente, Square::from_int(i as u8).unwrap())),
    konst::array::from_fn!(|i| lance_mask(Side::Gote, Square::from_int(i as u8).unwrap())),
];

const fn lance_mask(side: Side, sq: Square) -> Bitboard {
    let ranks = match side {
        Side::Sente => Bitboard::FULL.0 >> (9 * (9 - sq.rank() as u8)),
        Side::Gote => Bitboard::FULL.0 << (9 * (1 + sq.rank() as u8)),
    };
    Bitboard(sq.file().mask().0 & ranks)
}
