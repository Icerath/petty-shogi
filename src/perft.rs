use std::ops::ControlFlow;

use crate::board::Board;

impl Board {
    #[expect(clippy::missing_panics_doc)]
    #[must_use]
    pub fn perft(&self, depth: u32) -> u64 {
        self.try_perft(depth, &mut |_| false).continue_value().unwrap()
    }

    pub fn try_perft(
        &self,
        depth: u32,
        stop: &mut impl FnMut(&Board) -> bool,
    ) -> ControlFlow<(), u64> {
        if stop(self) {
            return ControlFlow::Break(());
        }
        match depth {
            0 => return ControlFlow::Continue(1),
            1 => return ControlFlow::Continue(self.legal_moves(0u64)),
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
        for (depth, expected) in (1..).zip([30, 900, 25470, 719_731, 19_861_490, 547_581_517]) {
            assert_eq!(Board::start_pos().perft(depth), expected, "at depth {depth}");
        }
    }
}
