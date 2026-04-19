use std::fmt::{self, Write};

use crate::Side;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Piece {
    SentePawn,
    GotePawn,
    SentePromotedPawn,
    GotePromotedPawn,
    SenteLance,
    GoteLance,
    SentePromotedLance,
    GotePromotedLance,
    SenteKnight,
    GoteKnight,
    SentePromotedKnight,
    GotePromotedKnight,
    SenteSilver,
    GoteSilver,
    SentePromotedSilver,
    GotePromotedSilver,
    SenteGold,
    GoteGold,
    _SentePromotedGold,
    _GotePromotedGold,
    SenteBishop,
    GoteBishop,
    SentePromotedBishop,
    GotePromotedBishop,
    SenteRook,
    GoteRook,
    SentePromotedRook,
    GotePromotedRook,
    SenteKing,
    GoteKing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PieceKind {
    Pawn,
    Lance,
    Knight,
    Silver,
    Gold,
    Bishop,
    Rook,
    King,
}

impl Piece {
    pub fn new(side: Side, kind: PieceKind, promoted: bool) -> Self {
        unsafe { Self::from_int_unchecked(side as u8 | (promoted as u8) << 1 | (kind as u8) << 2) }
    }

    pub unsafe fn from_int_unchecked(int: u8) -> Self {
        unsafe { std::mem::transmute(int) }
    }

    pub fn side(self) -> Side {
        Side::from_bool(self as u8 & 1 == 1)
    }

    pub fn promoted(self) -> bool {
        self as u8 & 2 == 2
    }

    pub fn kind(self) -> PieceKind {
        unsafe { PieceKind::from_int_unchecked((self as u8) >> 2) }
    }
}

impl PieceKind {
    pub const ALL: [Self; Self::LEN] = [
        Self::Pawn,
        Self::Lance,
        Self::Knight,
        Self::Silver,
        Self::Gold,
        Self::Bishop,
        Self::Rook,
        Self::King,
    ];
    pub const LEN: usize = Self::King as usize + 1;

    pub unsafe fn from_int_unchecked(int: u8) -> Self {
        unsafe { std::mem::transmute(int) }
    }
}

impl PieceKind {
    pub fn symbol(self) -> u8 {
        match self {
            Self::Pawn => b'p',
            Self::Lance => b'l',
            Self::Knight => b'n',
            Self::Silver => b's',
            Self::Gold => b'g',
            Self::Bishop => b'b',
            Self::Rook => b'r',
            Self::King => b'k',
        }
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.promoted() {
            write!(f, "+")?;
        }
        let mut c = self.kind().symbol();
        if self.side() == Side::Sente {
            c.make_ascii_uppercase();
        }
        f.write_char(c as char)
    }
}

impl Piece {
    pub fn try_from_symbol(symbol: u8) -> Option<Self> {
        Some(match symbol {
            b'p' => Self::GotePawn,
            b'P' => Self::SentePawn,
            b'l' => Self::GoteLance,
            b'L' => Self::SenteLance,
            b'n' => Self::GoteKnight,
            b'N' => Self::SenteKnight,
            b's' => Self::GoteSilver,
            b'S' => Self::SenteSilver,
            b'g' => Self::GoteGold,
            b'G' => Self::SenteGold,
            b'b' => Self::GoteBishop,
            b'B' => Self::SenteBishop,
            b'r' => Self::GoteRook,
            b'R' => Self::SenteRook,
            b'k' => Self::GoteKing,
            b'K' => Self::SenteKing,
            _ => return None,
        })
    }
}
