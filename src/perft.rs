use crate::board::Board;

impl Board {
    pub fn perft(&mut self, depth: u32) -> u32 {
        if depth == 1 {
            let mut count = 0;
            self.pseudolegal_moves(&mut count);
            return count as u32;
        }
        let mut sum = 0;
        self.clone().pseudolegal_moves(&mut |mov| {
            let copy = self.clone();
            self.play(mov);
            sum += self.perft(depth - 1);
            *self = copy;
        });
        sum
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
