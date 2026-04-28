use std::ops::ControlFlow;

use crate::board::Board;

impl Board {
    pub fn perft(&self, depth: u32) -> u64 {
        self.try_perft(depth, &mut || false).continue_value().unwrap()
    }

    pub fn try_perft(&self, depth: u32, stop: &mut impl FnMut() -> bool) -> ControlFlow<(), u64> {
        if stop() {
            return ControlFlow::Break(());
        }
        match depth {
            0 => return ControlFlow::Continue(1),
            1 => return ControlFlow::Continue(self.legal_moves(0)),
            _ => {}
        }
        let mut sum = 0;
        self.legal_moves(|mov| {
            let mut board = self.clone();
            board.play(mov);
            sum += board.try_perft(depth - 1, stop)?;
            ControlFlow::Continue(())
        })?;
        ControlFlow::Continue(sum)
    }
}

#[cfg(test)]
mod tests {
    use crate::board::Board;

    #[test]
    fn start() {
        for (depth, expected) in (1..).zip([30, 900, 25470, 719731, 19861490, 547581517]) {
            assert_eq!(Board::start_pos().perft(depth), expected, "at depth {depth}");
        }
    }
}
