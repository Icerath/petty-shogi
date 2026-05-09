use crate::{Bitboard, Board, Move, PieceKind, Square, Try, ptry};

pub trait Receiver {
    type Result: Try;
    type Output;

    fn recv_move(&mut self, mov: Move) -> Self::Result;
    fn finish(self, result: Self::Result) -> Self::Output;

    fn recv(&mut self, from: Square, squares: Bitboard, promote: bool) -> Self::Result {
        for sq in squares {
            ptry!(self.recv_move(Move::Board { from, to: sq, promoted: promote }));
        }
        Self::Result::output()
    }

    fn recv_drop(&mut self, piece: PieceKind, squares: Bitboard) -> Self::Result {
        for sq in squares {
            ptry!(self.recv_move(Move::Drop { piece, to: sq }));
        }
        Self::Result::output()
    }
}

impl<F, Output> Receiver for F
where
    F: FnMut(Move) -> Output,
    Output: Try,
{
    type Output = Output;
    type Result = Output;

    fn recv_move(&mut self, mov: Move) -> Output {
        (*self)(mov)
    }

    fn finish(self, result: Self::Result) -> Self::Output {
        result
    }
}

impl Receiver for Vec<Move> {
    type Output = Self;
    type Result = ();

    fn recv_move(&mut self, mov: Move) {
        self.push(mov);
    }

    fn finish(self, (): ()) -> Self::Output {
        self
    }
}

impl Receiver for &mut Vec<Move> {
    type Output = ();
    type Result = ();

    fn recv_move(&mut self, mov: Move) {
        (*self).recv_move(mov);
    }

    fn finish(self, (): ()) {}
}

macro_rules! recv_int {
    ($($int:ty),* $(,)?) => {
        $(impl Receiver for $int {
            type Output = Self;
            type Result = ();

            fn recv_move(&mut self, _: Move) {
                *self += 1;
            }

            fn finish(self, (): ()) -> Self::Output {
                self
            }

            fn recv(&mut self, _: Square, bb: Bitboard, _: bool) {
                *self += bb.count() as $int;
            }
        })*
    };
}
recv_int!(u16, u32, u64, u128, i16, i32, i64, i128);

impl Receiver for Bitboard {
    type Output = Self;
    type Result = ();

    fn recv_move(&mut self, mov: Move) -> Self::Result {
        self.insert(mov.to());
    }

    fn recv(&mut self, _: Square, squares: Bitboard, _: bool) -> Self::Result {
        *self |= squares;
    }

    fn recv_drop(&mut self, _: PieceKind, squares: Bitboard) -> Self::Result {
        *self |= squares;
    }

    fn finish(self, (): ()) -> Self {
        self
    }
}

pub struct Legal<'a, R> {
    pub board: &'a Board,
    pub recv: R,
}

impl<R: Receiver> Receiver for Legal<'_, R> {
    type Output = R::Output;
    type Result = R::Result;

    fn recv_move(&mut self, mov: Move) -> Self::Result {
        if self.board.is_legal(mov) { self.recv.recv_move(mov) } else { Self::Result::output() }
    }

    fn finish(self, result: Self::Result) -> Self::Output {
        self.recv.finish(result)
    }
}
