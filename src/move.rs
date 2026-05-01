use std::{fmt, str::FromStr};

use crate::{PieceKind, Square};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Move {
    Board { from: Square, to: Square, promoted: bool },
    Drop { piece: PieceKind, to: Square },
}

impl Move {
    pub const PLACEHOLDER: Self = Self::Board { from: Square::A1, to: Square::A1, promoted: false };

    #[must_use]
    pub const fn to(self) -> Square {
        match self {
            Self::Drop { to, .. } | Self::Board { to, .. } => to,
        }
    }
}

#[derive(Debug)]
pub struct InvalidMoveStr;

impl fmt::Display for InvalidMoveStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid move")
    }
}

impl FromStr for Move {
    type Err = InvalidMoveStr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.as_bytes();
        if s.len() < 4 {
            return Err(InvalidMoveStr);
        }
        if s.len() > 5 {
            return Err(InvalidMoveStr);
        }
        if s[1] == b'*' {
            let piece = match s[0] {
                b'P' => PieceKind::Pawn,
                b'L' => PieceKind::Lance,
                b'N' => PieceKind::Knight,
                b'S' => PieceKind::Silver,
                b'G' => PieceKind::Gold,
                b'B' => PieceKind::Bishop,
                b'R' => PieceKind::Rook,
                _ => return Err(InvalidMoveStr),
            };
            let to = Square::parse(s[2..=3].try_into().unwrap()).ok_or(InvalidMoveStr)?;
            Ok(Self::Drop { piece, to })
        } else {
            let from = Square::parse(s[0..=1].try_into().unwrap()).ok_or(InvalidMoveStr)?;
            let to = Square::parse(s[2..=3].try_into().unwrap()).ok_or(InvalidMoveStr)?;
            let mut promoted = false;
            if let Some(&c) = s.get(4) {
                if c != b'+' {
                    return Err(InvalidMoveStr);
                }
                promoted = true;
            }
            Ok(Self::Board { from, to, promoted })
        }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Board { from, to, promoted } => {
                write!(f, "{from}{to}")?;
                if *promoted {
                    write!(f, "+")?;
                }
                Ok(())
            }
            Self::Drop { piece, to } => {
                let piece_symbol = match piece {
                    PieceKind::Pawn => b'P',
                    PieceKind::Lance => b'L',
                    PieceKind::Knight => b'N',
                    PieceKind::Silver => b'S',
                    PieceKind::Gold => b'G',
                    PieceKind::Bishop => b'B',
                    PieceKind::Rook => b'R',
                    PieceKind::King => b'K',
                };
                write!(f, "{}*{to}", piece_symbol as char)
            }
        }
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Move {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str = <&str>::deserialize(deserializer)?;
        Self::from_str(str).map_err(serde::de::Error::custom)
    }
}
#[cfg(feature = "serde")]
impl serde::Serialize for Move {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        assert_eq!(
            Move::from_str("1a9i+").unwrap(),
            Move::Board { from: Square::A1, to: Square::I9, promoted: true }
        );
    }
}
