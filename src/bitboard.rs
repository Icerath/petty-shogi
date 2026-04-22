use std::{
    fmt::{self, Write as _},
    ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not},
};

use konst::array::from_fn;

use crate::{File, Rank, Side, Square, bitboard};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub struct Bitboard(u128);

impl Bitboard {
    pub const EMPTY: Self = Self(0);
    pub const FULL: Self = Self(u128::MAX >> (128 - 81));

    #[must_use]
    pub const fn from_square(sq: Square) -> Self {
        const LUT: [Bitboard; 81] = from_fn!(|i| Bitboard(1u128 << i));
        LUT[sq as usize]
    }

    #[must_use]
    pub const fn from_rank(rank: Rank) -> Self {
        const RANK0: Bitboard = bitboard! {
            1 1 1 1 1 1 1 1 1
            0 0 0 0 0 0 0 0 0
            0 0 0 0 0 0 0 0 0
            0 0 0 0 0 0 0 0 0
            0 0 0 0 0 0 0 0 0
            0 0 0 0 0 0 0 0 0
            0 0 0 0 0 0 0 0 0
            0 0 0 0 0 0 0 0 0
            0 0 0 0 0 0 0 0 0
        };
        const LUT: [Bitboard; 9] = from_fn!(|rank| Bitboard(RANK0.0 << (rank as u128 * 9)));
        LUT[rank as usize]
    }

    #[must_use]
    pub const fn from_file(file: File) -> Self {
        const FILE0: Bitboard = bitboard! {
            1 0 0 0 0 0 0 0 0
            1 0 0 0 0 0 0 0 0
            1 0 0 0 0 0 0 0 0
            1 0 0 0 0 0 0 0 0
            1 0 0 0 0 0 0 0 0
            1 0 0 0 0 0 0 0 0
            1 0 0 0 0 0 0 0 0
            1 0 0 0 0 0 0 0 0
            1 0 0 0 0 0 0 0 0
        };
        const LUT: [Bitboard; 9] = from_fn!(|file| Bitboard(FILE0.0 << (file as u128)));
        LUT[file as usize]
    }

    #[must_use]
    pub const fn count(self) -> u32 {
        self.0.count_ones()
    }

    pub fn for_each(mut self, mut f: impl FnMut(Square)) {
        while let Some(next) = self.bitscan() {
            f(next);
            self.bitscan_pop();
        }
    }

    #[must_use]
    pub const fn bitscan(self) -> Option<Square> {
        if self.is_empty() { None } else { Some(unsafe { self.bitscan_unchecked() }) }
    }

    /// # Safety
    /// `self.is_empty()` must be false
    #[must_use]
    pub const unsafe fn bitscan_unchecked(self) -> Square {
        unsafe { Square::from_int_unchecked(self.0.trailing_zeros() as u8) }
    }

    pub const fn bitscan_pop(&mut self) {
        self.0 &= self.0 - 1;
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn contains(self, sq: Square) -> bool {
        !(self & sq.mask()).is_empty()
    }

    pub fn remove(&mut self, sq: Square) {
        *self &= !sq.mask();
    }

    pub const fn insert(&mut self, sq: Square) {
        self.0 |= sq.mask().0;
    }

    #[must_use]
    /// slow, should not be used in hot path
    pub const fn flip(mut self) -> Self {
        let mut new = Bitboard::EMPTY;
        while let Some(sq) = self.bitscan() {
            new.insert(sq.flip());
            self.bitscan_pop();
        }
        new
    }

    pub const fn from_bits(bits: [bool; 81]) -> Self {
        let mut bb = Bitboard::EMPTY;
        let mut i = 0;
        while i < 81 {
            if bits[i as usize] {
                bb.insert(Square::from_int(i).unwrap());
            }
            i += 1;
        }
        bb
    }

    pub const fn shift_forward(self, side: Side) -> Self {
        match side {
            Side::Sente => self.shift_up(),
            Side::Gote => self.shift_down(),
        }
    }

    pub const fn shift_back(self, side: Side) -> Self {
        match side {
            Side::Sente => self.shift_down(),
            Side::Gote => self.shift_up(),
        }
    }

    pub const fn shift_up(self) -> Self {
        Self(self.0 >> 9)
    }

    pub const fn shift_down(self) -> Self {
        Self((self.0 << 9) & Self::FULL.0)
    }

    // FIXME: replace with const traits
    pub const fn bitand(self, rhs: Bitboard) -> Self {
        Self(self.0 & rhs.0)
    }

    pub const fn iter(self) -> Iter {
        Iter(self)
    }
}

impl BitOr for Bitboard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bitboard {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

impl BitAnd for Bitboard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for Bitboard {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}

impl Not for Bitboard {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0) & Self::FULL
    }
}

impl fmt::Debug for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for sq in Square::ALL {
            f.write_char(if self.contains(sq) { '1' } else { '0' })?;
        }
        Ok(())
    }
}

impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for sq in Square::ALL {
            write!(f, "{} ", self.contains(sq) as u8)?;
            if sq.file().right().is_none() {
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

pub struct Iter(pub Bitboard);

impl IntoIterator for Bitboard {
    type IntoIter = Iter;
    type Item = Square;

    fn into_iter(self) -> Self::IntoIter {
        Iter(self)
    }
}

impl Iterator for Iter {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.0.bitscan()?;
        self.0.bitscan_pop();
        Some(next)
    }
}

#[macro_export]
macro_rules! _bit {
    (0) => {
        false
    };
    (1) => {
        true
    };
}

#[macro_export]
macro_rules! bitboard {
    {$($bit:tt)*} => {
        const { $crate::bitboard::Bitboard::from_bits([$($crate::_bit!($bit),)*]) }
    };
}

#[cfg(test)]
mod tests {
    use crate::{bitboard::Bitboard, square::Square};

    #[test]
    fn full81() {
        assert_eq!(Bitboard::FULL.count(), 81);
    }

    #[test]
    fn flip() {
        assert_eq!(Bitboard::EMPTY, Bitboard::EMPTY);
        assert_eq!(Bitboard::FULL, Bitboard::FULL);
        assert_eq!(Square::A1.mask().flip(), Square::I1.mask());
    }
}
