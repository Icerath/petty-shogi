use std::{fmt, str::FromStr};

use crate::{PieceKind, Square};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Action {
    Move { from: Square, to: Square, promoted: bool },
    Drop { piece: PieceKind, to: Square },
}

impl Action {
    pub const PLACEHOLDER: Self = Self::Move { from: Square::A1, to: Square::A1, promoted: false };

    #[must_use]
    pub const fn to(self) -> Square {
        match self {
            Self::Drop { to, .. } | Self::Move { to, .. } => to,
        }
    }
}

#[derive(Debug)]
pub struct InvalidActionStr;

impl fmt::Display for InvalidActionStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid action")
    }
}

impl FromStr for Action {
    type Err = InvalidActionStr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.as_bytes();
        if s.len() < 4 {
            return Err(InvalidActionStr);
        }
        if s.len() > 5 {
            return Err(InvalidActionStr);
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
                _ => return Err(InvalidActionStr),
            };
            let to = Square::parse(s[2..=3].try_into().unwrap()).ok_or(InvalidActionStr)?;
            Ok(Self::Drop { piece, to })
        } else {
            let from = Square::parse(s[0..=1].try_into().unwrap()).ok_or(InvalidActionStr)?;
            let to = Square::parse(s[2..=3].try_into().unwrap()).ok_or(InvalidActionStr)?;
            let mut promoted = false;
            if let Some(&c) = s.get(4) {
                if c != b'+' {
                    return Err(InvalidActionStr);
                }
                promoted = true;
            }
            Ok(Self::Move { from, to, promoted })
        }
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Move { from, to, promoted } => {
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

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Action {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str = <&str>::deserialize(deserializer)?;
        Self::from_str(str).map_err(serde::de::Error::custom)
    }
}
#[cfg(feature = "serde")]
impl serde::Serialize for Action {
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
            Action::from_str("1a9i+").unwrap(),
            Action::Move { from: Square::A1, to: Square::I9, promoted: true }
        );
    }
}
