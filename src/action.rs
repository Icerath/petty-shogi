use std::fmt;

use crate::{Piece, Square};

#[derive(Clone, Copy)]
pub enum Action {
    Move { from: Square, to: Square, promoted: bool },
    Drop { piece: Piece, to: Square },
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
            Self::Drop { piece, to } => write!(f, "{piece}*{to}"),
        }
    }
}

impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}
