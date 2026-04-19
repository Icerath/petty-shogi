use std::ops::Not;

use crate::{Bitboard, Rank, bitboard};

// consider using 1 and -1 for different codegen?
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Side {
    #[default]
    Sente, // black
    Gote, // white
}

impl Side {
    pub fn from_bool(gote: bool) -> Self {
        if gote { Self::Gote } else { Self::Sente }
    }

    pub fn forward(self) -> i8 {
        match self {
            Self::Sente => -1,
            Self::Gote => 1,
        }
    }

    pub fn end_rank(self) -> Rank {
        match self {
            Self::Sente => Rank::A,
            Self::Gote => Rank::I,
        }
    }

    pub const fn promotion_zone(self) -> Bitboard {
        match self {
            Self::Sente => SENTE_PROMOTION_ZONE,
            Self::Gote => const { SENTE_PROMOTION_ZONE.flip() },
        }
    }
}

const SENTE_PROMOTION_ZONE: Bitboard = bitboard! {
    1 1 1 1 1 1 1 1 1
    1 1 1 1 1 1 1 1 1
    1 1 1 1 1 1 1 1 1
    0 0 0 0 0 0 0 0 0
    0 0 0 0 0 0 0 0 0
    0 0 0 0 0 0 0 0 0
    0 0 0 0 0 0 0 0 0
    0 0 0 0 0 0 0 0 0
    0 0 0 0 0 0 0 0 0
};

impl Not for Side {
    type Output = Self;

    fn not(self) -> Self::Output {
        match self {
            Self::Sente => Self::Gote,
            Self::Gote => Self::Sente,
        }
    }
}
