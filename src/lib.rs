mod action;
mod bitboard;
mod board;
mod engine;
mod index;
mod movegen;
mod perft;
mod piece;
mod sfen;
mod side;
mod square;
mod zobrist;

use std::ops::ControlFlow;

pub use action::Action;
pub use bitboard::Bitboard;
pub use board::{Board, Hand};
pub use engine::{Engine, command, response};
pub use piece::{Piece, PieceKind};
pub use side::Side;
pub use square::{File, Rank, Square};

/// Similar to the currently unstable `std::ops::Try`, but also implemented for `()`.
// I really really wish the std Try trait worked here and was stable, but too bad.
pub trait Try {
    type Residual;

    fn output() -> Self;
    fn from_residual(residual: Self::Residual) -> Self;
    fn branch(self) -> ControlFlow<Self::Residual>;
}

impl Try for () {
    type Residual = core::convert::Infallible;

    fn from_residual(_: Self::Residual) -> Self {}

    fn branch(self) -> ControlFlow<Self::Residual> {
        ControlFlow::Continue(())
    }

    fn output() -> Self {}
}

impl<T> Try for ControlFlow<T> {
    type Residual = T;

    fn from_residual(residual: Self::Residual) -> Self {
        Self::Break(residual)
    }

    fn branch(self) -> ControlFlow<Self::Residual> {
        self
    }

    fn output() -> Self {
        Self::Continue(())
    }
}

impl<E> Try for Result<(), E> {
    type Residual = E;

    fn from_residual(residual: Self::Residual) -> Self {
        Err(residual)
    }

    fn branch(self) -> ControlFlow<Self::Residual> {
        match self {
            Ok(()) => ControlFlow::Continue(()),
            Err(e) => ControlFlow::Break(e),
        }
    }

    fn output() -> Self {
        Ok(())
    }
}

#[macro_export]
macro_rules! ptry {
    ($expr:expr) => {
        match $expr.branch() {
            ::core::ops::ControlFlow::Break(b) => return $crate::Try::from_residual(b),
            ::core::ops::ControlFlow::Continue(c) => c,
        }
    };
}
