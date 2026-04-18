use crate::board::Board;

impl Board {
    pub fn perft(&mut self, depth: u64) -> u64 {
        if depth == 0 {
            return 1;
        }
        if depth == 1 {
            return *self.legal_moves(&mut 0);
        }
        let mut sum = 0;
        self.clone().legal_moves(&mut |mov| {
            let copy = self.clone();
            self.play(mov);
            sum += self.perft(depth - 1);
            *self = copy;
        });
        sum
    }

    pub fn print_perft(&mut self, depth: u64) -> u64 {
        let mut sum = 0;
        self.clone().legal_moves(&mut |mov| {
            let copy = self.clone();
            self.play(mov);
            let positions = self.perft(depth - 1);
            println!("{mov}: {positions}");
            sum += positions;
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
