use crate::{Action, Board, Try};

pub trait Receiver {
    type Result: Try;
    type Output;

    fn recv(&mut self, action: Action) -> Self::Result;
    fn finish(self, result: Self::Result) -> Self::Output;
}

impl<F, Output> Receiver for F
where
    F: FnMut(Action) -> Output,
    Output: Try,
{
    type Output = Output;
    type Result = Output;

    fn recv(&mut self, action: Action) -> Output {
        (*self)(action)
    }

    fn finish(self, result: Self::Result) -> Self::Output {
        result
    }
}

impl Receiver for Vec<Action> {
    type Output = Vec<Action>;
    type Result = ();

    fn recv(&mut self, action: Action) {
        self.push(action);
    }

    fn finish(self, _: Self::Result) -> Self::Output {
        self
    }
}

impl Receiver for &mut Vec<Action> {
    type Output = ();
    type Result = ();

    fn recv(&mut self, action: Action) {
        (*self).recv(action)
    }

    fn finish(self, _: Self::Result) {}
}

impl Receiver for u64 {
    type Output = u64;
    type Result = ();

    fn recv(&mut self, _: Action) {
        *self += 1;
    }

    fn finish(self, _: Self::Result) -> Self::Output {
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

    fn recv(&mut self, action: Action) -> Self::Result {
        if self.board.is_legal(action) { self.recv.recv(action) } else { Self::Result::output() }
    }

    fn finish(self, result: Self::Result) -> Self::Output {
        self.recv.finish(result)
    }
}
