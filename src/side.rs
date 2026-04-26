use std::ops::Not;

use crate::{Bitboard, Rank, bitboard};

// consider using 1 and -1 for different codegen?
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Side {
    #[default]
    Sente, // black
    Gote, // white
}

#[cfg(feature = "serde")]
impl serde::Serialize for Side {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            Self::Sente => "b",
            Self::Gote => "w",
        };
        s.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Side {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <&str>::deserialize(deserializer)?;
        match s {
            "w" => Ok(Side::Gote),
            "b" => Ok(Side::Sente),
            _ => Err(serde::de::Error::custom("invalid side")),
        }
    }
}

impl Side {
    pub const LEN: usize = 2;

    pub fn from_bool(gote: bool) -> Self {
        if gote { Self::Gote } else { Self::Sente }
    }

    pub const fn forward(self) -> i8 {
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

    pub const fn flip(self) -> Self {
        match self {
            Self::Sente => Self::Gote,
            Self::Gote => Self::Sente,
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
