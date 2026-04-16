use std::fmt;

use crate::{bitboard::Bitboard, side::Side};

#[derive(Debug, Clone, Copy)]
#[rustfmt::skip]
pub enum Square {
    A9, A8, A7, A6, A5, A4, A3, A2, A1,
    B9, B8, B7, B6, B5, B4, B3, B2, B1,
    C9, C8, C7, C6, C5, C4, C3, C2, C1,
    D9, D8, D7, D6, D5, D4, D3, D2, D1,
    E9, E8, E7, E6, E5, E4, E3, E2, E1,
    F9, F8, F7, F6, F5, F4, F3, F2, F1,
    G9, G8, G7, G6, G5, G4, G3, G2, G1,
    H9, H8, H7, H6, H5, H4, H3, H2, H1,
    I9, I8, I7, I6, I5, I4, I3, I2, I1,
}

#[derive(Debug, Clone, Copy)]
#[rustfmt::skip]
pub enum File {
    _9, _8, _7, _6, _5, _4, _3, _2, _1,
}

#[derive(Debug, Clone, Copy)]
#[rustfmt::skip]
#[expect(unused)]
pub enum Rank {
    A, B, C, D, E, F, G, H, I,
}

impl Square {
    pub const ALL: [Self; 81] = {
        let mut out = [Square::A1; 81];
        let mut i = 0;
        while i < 81 {
            out[i as usize] = Square::from_int(i).unwrap();
            i += 1;
        }
        out
    };

    pub const fn new(file: File, rank: Rank) -> Self {
        unsafe { Self::from_int_unchecked(file as u8 + rank as u8 * 9) }
    }

    pub const unsafe fn from_int_unchecked(int: u8) -> Self {
        debug_assert!(int < 81);
        unsafe { std::mem::transmute(int) }
    }

    pub const fn from_int(int: u8) -> Option<Self> {
        if int < 81 { Some(unsafe { Self::from_int_unchecked(int) }) } else { None }
    }

    pub const fn as_str(self) -> &'static str {
        #[rustfmt::skip]
        const NAMES: [&str; 81] = [
            "a9", "a8", "a7", "a6", "a5", "a4", "a3", "a2", "a1",
            "b9", "b8", "b7", "b6", "b5", "b4", "b3", "b2", "b1",
            "c9", "c8", "c7", "c6", "c5", "c4", "c3", "c2", "c1",
            "d9", "d8", "d7", "d6", "d5", "d4", "d3", "d2", "d1",
            "e9", "e8", "e7", "e6", "e5", "e4", "e3", "e2", "e1",
            "f9", "f8", "f7", "f6", "f5", "f4", "f3", "f2", "f1",
            "g9", "g8", "g7", "g6", "g5", "g4", "g3", "g2", "g1",
            "h9", "h8", "h7", "h6", "h5", "h4", "h3", "h2", "h1",
            "i9", "i8", "i7", "i6", "i5", "i4", "i3", "i2", "i1",
        ];
        NAMES[self as usize]
    }

    pub const fn mask(self) -> Bitboard {
        Bitboard::from_square(self)
    }

    pub const fn file(self) -> File {
        unsafe { std::mem::transmute(self as u8 % 9) }
    }

    pub const fn rank(self) -> Rank {
        unsafe { std::mem::transmute(self as u8 / 9) }
    }

    pub const fn is_promotion_zone(self, side: Side) -> bool {
        self.rank().is_promotion_zone(side)
    }

    pub const fn flip(self) -> Self {
        use Square::*;
        #[rustfmt::skip]
        const FLIPPED: [Square; 81] = [
            I9, I8, I7, I6, I5, I4, I3, I2, I1,
            H9, H8, H7, H6, H5, H4, H3, H2, H1,
            G9, G8, G7, G6, G5, G4, G3, G2, G1,
            F9, F8, F7, F6, F5, F4, F3, F2, F1,
            E9, E8, E7, E6, E5, E4, E3, E2, E1,
            D9, D8, D7, D6, D5, D4, D3, D2, D1,
            C9, C8, C7, C6, C5, C4, C3, C2, C1,
            B9, B8, B7, B6, B5, B4, B3, B2, B1,
            A9, A8, A7, A6, A5, A4, A3, A2, A1,
        ];
        FLIPPED[self as usize]
    }

    pub const fn nforward(self, side: Side, n: u8) -> Option<Self> {
        let Some(rank) = self.rank().nforward(side, n) else { return None };
        Some(Self::new(self.file(), rank))
    }

    pub const fn nback(self, side: Side, n: u8) -> Option<Self> {
        let Some(rank) = self.rank().nback(side, n) else { return None };
        Some(Self::new(self.file(), rank))
    }

    pub const fn up(self) -> Option<Self> {
        let Some(rank) = self.rank().up() else { return None };
        Some(Self::new(self.file(), rank))
    }

    pub const fn forward(self, side: Side) -> Option<Self> {
        let Some(rank) = self.rank().forward(side) else { return None };
        Some(Self::new(self.file(), rank))
    }

    pub const fn back(self, side: Side) -> Option<Self> {
        let Some(rank) = self.rank().back(side) else { return None };
        Some(Self::new(self.file(), rank))
    }

    pub const fn down(self) -> Option<Self> {
        let Some(rank) = self.rank().down() else { return None };
        Some(Self::new(self.file(), rank))
    }

    pub const fn left(self) -> Option<Self> {
        let Some(file) = self.file().left() else { return None };
        Some(Self::new(file, self.rank()))
    }

    pub const fn right(self) -> Option<Self> {
        let Some(file) = self.file().right() else { return None };
        Some(Self::new(file, self.rank()))
    }

    pub const fn offset_file_rank(self, file: i8, rank: i8) -> Option<Self> {
        let file @ 0..9 = (self.file() as i8) + file else { return None };
        let rank @ 0..9 = (self.rank() as i8) + rank else { return None };
        Some(unsafe { Self::from_int_unchecked(file as u8 + rank as u8 * 9) })
    }
}

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_str(), f)
    }
}

#[expect(clippy::missing_transmute_annotations)]
impl File {
    #[inline(always)]
    pub const fn from_int(int: u8) -> Option<Self> {
        match int {
            0..9 => Some(unsafe { std::mem::transmute(int) }),
            _ => None,
        }
    }

    #[inline(always)]
    pub const fn left(self) -> Option<Self> {
        if self as u8 == 0 { None } else { unsafe { std::mem::transmute(self as u8 - 1) } }
    }

    #[inline(always)]
    pub const fn right(self) -> Option<Self> {
        if self as u8 == 8 { None } else { unsafe { std::mem::transmute(self as u8 + 1) } }
    }
}

#[expect(clippy::missing_transmute_annotations)]
impl Rank {
    pub const SYMBOLS: [u8; 9] = [b'a', b'b', b'c', b'd', b'e', b'f', b'g', b'h', b'i'];

    #[inline(always)]
    pub const fn from_int(int: u8) -> Option<Self> {
        match int {
            0..9 => Some(unsafe { std::mem::transmute(int) }),
            _ => None,
        }
    }

    #[inline(always)]
    pub const fn is_promotion_zone(self, side: Side) -> bool {
        match side {
            Side::Sente => (self as u8) < 3,
            Side::Gote => self as u8 > 5,
        }
    }

    pub const fn nforward(self, side: Side, n: u8) -> Option<Self> {
        match side {
            Side::Sente => self.nup(n),
            Side::Gote => self.ndown(n),
        }
    }

    pub const fn nback(self, side: Side, n: u8) -> Option<Self> {
        match side {
            Side::Sente => self.ndown(n),
            Side::Gote => self.nup(n),
        }
    }

    pub const fn nup(self, n: u8) -> Option<Self> {
        if (self as u8) < n { None } else { unsafe { std::mem::transmute(self as u8 - n) } }
    }

    pub const fn ndown(self, n: u8) -> Option<Self> {
        if (self as u8) + n >= 9 { None } else { unsafe { std::mem::transmute(self as u8 + n) } }
    }

    pub const fn up(self) -> Option<Self> {
        self.nup(1)
    }

    pub const fn down(self) -> Option<Self> {
        self.ndown(1)
    }

    pub const fn forward(self, side: Side) -> Option<Self> {
        self.nforward(side, 1)
    }

    pub const fn back(self, side: Side) -> Option<Self> {
        self.nback(side, 1)
    }

    pub const fn mask(self) -> Bitboard {
        Bitboard::from_rank(self)
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (Self::SYMBOLS[*self as usize] as char).fmt(f)
    }
}
