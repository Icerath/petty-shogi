use crate::{Board, Move, Try};

pub trait Receiver {
    type Result: Try;
    type Output;

    const PROMOTE_FILTER: Option<bool> = None;

    fn recv(&mut self, mov: Move) -> Self::Result;
    fn finish(self, result: Self::Result) -> Self::Output;
}

impl<F, Output> Receiver for F
where
    F: FnMut(Move) -> Output,
    Output: Try,
{
    type Output = Output;
    type Result = Output;

    fn recv(&mut self, mov: Move) -> Output {
        (*self)(mov)
    }

    fn finish(self, result: Self::Result) -> Self::Output {
        result
    }
}

impl Receiver for Vec<Move> {
    type Output = Self;
    type Result = ();

    fn recv(&mut self, mov: Move) {
        self.push(mov);
    }

    fn finish(self, (): ()) -> Self::Output {
        self
    }
}

impl Receiver for &mut Vec<Move> {
    type Output = ();
    type Result = ();

    fn recv(&mut self, mov: Move) {
        (*self).recv(mov);
    }

    fn finish(self, (): ()) {}
}

impl Receiver for u64 {
    type Output = Self;
    type Result = ();

    fn recv(&mut self, _: Move) {
        *self += 1;
    }

    fn finish(self, (): ()) -> Self::Output {
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

    const PROMOTE_FILTER: Option<bool> = R::PROMOTE_FILTER;

    fn recv(&mut self, mov: Move) -> Self::Result {
        if self.board.is_legal(mov) { self.recv.recv(mov) } else { Self::Result::output() }
    }

    fn finish(self, result: Self::Result) -> Self::Output {
        self.recv.finish(result)
    }
}

#[expect(unused)]
pub struct FilterPromote<const FILTER: bool, R>(pub R);

impl<const FILTER: bool, R: Receiver> Receiver for FilterPromote<FILTER, R> {
    type Output = R::Output;
    type Result = R::Result;

    const PROMOTE_FILTER: Option<bool> = Some(FILTER);

    fn recv(&mut self, mov: Move) -> Self::Result {
        self.0.recv(mov)
    }

    fn finish(self, result: Self::Result) -> Self::Output {
        self.0.finish(result)
    }
}
