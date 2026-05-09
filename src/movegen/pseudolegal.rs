use super::{
    Receiver,
    magic::{bishop_moves, lance_moves, rook_moves},
};
use crate::{Bitboard, Board, Move, PieceKind, Rank, Side, Square, Try, bitboard, ptry};

pub static KNIGHT_LUT: [[Bitboard; Square::LEN]; 2] =
    [compute_knight(Side::Sente), compute_knight(Side::Gote)];
pub static KNIGHT_PROMOTION_LUT: [[Bitboard; Square::LEN]; 2] =
    [compute_knight_promotion(Side::Sente), compute_knight_promotion(Side::Gote)];

pub static SILVER_LUT: [[Bitboard; Square::LEN]; 2] =
    [compute_silver(Side::Sente), compute_silver(Side::Gote)];

pub static GOLD_LUT: [[Bitboard; Square::LEN]; 2] =
    [compute_gold(Side::Sente), compute_gold(Side::Gote)];
pub static KING_LUT: [Bitboard; Square::LEN] = compute_king();

impl Board {
    pub(crate) fn pseudolegal_moves_<R: Receiver>(&self, r: &mut R, mask: Bitboard) -> R::Result {
        let checkers = match (self[PieceKind::King] & self[self.active]).bitscan() {
            Some(king_square) => self.gen_attackers(king_square, self.active),
            None => Bitboard::EMPTY,
        };
        macro_rules! moves {
            ($($ident:ident),* $(,)?) => {
                $(ptry!(self.$ident(mask, r)));*
            };
        }

        if checkers.count() < 2 {
            moves!(
                pawn_moves,
                lance_moves,
                knight_moves,
                silver_moves,
                gold_moves,
                unpromoted_bishop_moves,
                promoted_bishop_moves,
                unpromoted_rook_moves,
                promoted_rook_moves,
                drop_moves,
            );
        }
        self.king_moves(mask, r)
    }

    pub(crate) fn is_pseudolegal(&self, mov: Move) -> bool {
        match mov {
            Move::Board { from, to, promoted } => self.is_pseudolegal_board(from, to, promoted),
            Move::Drop { piece, to } => self.is_pseudolegal_drop(piece, to),
        }
    }

    fn is_pseudolegal_board(&self, from: Square, to: Square, promote: bool) -> bool {
        let Some(from_piece) = self.pieces.get(from) else {
            return false;
        };
        if from_piece.side() != self.active {
            return false;
        }
        let mask = !self[self.active];
        let mut bb = Bitboard::EMPTY;

        if promote && !from.is_promotion_zone(self.active) && !to.is_promotion_zone(self.active) {
            return false;
        }

        if promote && let PieceKind::Gold | PieceKind::King = from_piece.kind() {
            return false;
        }

        if promote && from_piece.promoted() {
            return false;
        }

        if !promote {
            if let PieceKind::Pawn | PieceKind::Lance = from_piece.kind()
                && to.rank() == self.active.end_rank()
            {
                return false;
            }
            if from_piece.kind() == PieceKind::Knight
                && (to.rank() == self.active.end_rank()
                    || to.rank() == self.active.end_rank().back(self.active).unwrap())
            {
                return false;
            }
        }

        match (from_piece.kind(), from_piece.promoted()) {
            (PieceKind::Pawn, false) => {
                if let Some(forward) = from.forward(self.active) {
                    bb.insert(forward);
                }
            }
            (PieceKind::Lance, false) => lance(self, mask, from, &mut bb),
            (PieceKind::Knight, false) => knight(self, mask, from, &mut bb),
            (PieceKind::Silver, false) => silver(self, mask, from, &mut bb),
            (PieceKind::Gold, _) => gold(self, mask, from, &mut bb),
            (PieceKind::Bishop, false) => bishop::<false, _>(self, mask, from, &mut bb),
            (PieceKind::Rook, false) => rook::<false, _>(self, mask, from, &mut bb),
            (PieceKind::King, _) => king(mask, from, &mut bb),
            (PieceKind::Pawn | PieceKind::Lance | PieceKind::Knight | PieceKind::Silver, true) => {
                gold(self, mask, from, &mut bb);
            }
            (PieceKind::Bishop, true) => bishop::<true, _>(self, mask, from, &mut bb),
            (PieceKind::Rook, true) => rook::<true, _>(self, mask, from, &mut bb),
        }
        bb.contains(to)
    }

    fn is_pseudolegal_drop(&self, piece: PieceKind, to: Square) -> bool {
        if self.hands[self.active][piece] == 0 {
            return false;
        }
        let empty_squares = !self.pieces.all();
        if !empty_squares.contains(to) {
            return false;
        }
        if let PieceKind::Pawn | PieceKind::Lance | PieceKind::Knight = piece
            && to.rank() == self.active.end_rank()
        {
            return false;
        }

        if piece == PieceKind::Knight
            && to.rank() == self.active.end_rank().back(self.active).unwrap()
        {
            return false;
        }

        if piece == PieceKind::Pawn
            && !(self[PieceKind::Pawn] & self[self.active] & to.file().mask()).is_empty()
        {
            return false;
        }
        true
    }

    fn drop_moves<R: Receiver>(&self, mask: Bitboard, r: &mut R) -> R::Result {
        let empty_squares = mask & !self[!self.active];

        if empty_squares.is_empty() {
            return R::Result::output();
        }

        // pawns
        if self.hands[self.active][PieceKind::Pawn] > 0 {
            let mut busy_files = Bitboard::EMPTY;
            for sq in self[PieceKind::Pawn] & !self.pieces.promoted & self[self.active] {
                busy_files |= sq.file().mask();
            }
            ptry!(r.recv_drop(
                PieceKind::Pawn,
                empty_squares & !self.active.end_rank().mask() & !busy_files
            ));
        }
        // lances
        if self.hands[self.active][PieceKind::Lance] > 0 {
            ptry!(r.recv_drop(PieceKind::Lance, empty_squares & !self.active.end_rank().mask()));
        }
        // knights
        if self.hands[self.active][PieceKind::Knight] > 0 {
            const MASKS: [Bitboard; 2] =
                [Bitboard::FULL.shift_down().shift_down(), Bitboard::FULL.shift_up().shift_up()];
            ptry!(r.recv_drop(PieceKind::Knight, empty_squares & MASKS[self.active]));
        }
        // rest
        for &piece in &PieceKind::ALL[PieceKind::Silver as usize..PieceKind::King as usize] {
            if self.hands[self.active][piece] == 0 {
                continue;
            }
            ptry!(r.recv_drop(piece, empty_squares));
        }
        R::Result::output()
    }

    fn pawn_moves<R: Receiver>(&self, mask: Bitboard, r: &mut R) -> R::Result {
        const SENTE_PROMOTE: Bitboard = bitboard! {
            0 0 0 0 0 0 0 0 0
            1 1 1 1 1 1 1 1 1
            1 1 1 1 1 1 1 1 1
            1 1 1 1 1 1 1 1 1
            0 0 0 0 0 0 0 0 0
            0 0 0 0 0 0 0 0 0
            0 0 0 0 0 0 0 0 0
            0 0 0 0 0 0 0 0 0
            0 0 0 0 0 0 0 0 0
        };
        const SENTE_NOPROMOTE: Bitboard = bitboard! {
            0 0 0 0 0 0 0 0 0
            0 0 0 0 0 0 0 0 0
            1 1 1 1 1 1 1 1 1
            1 1 1 1 1 1 1 1 1
            1 1 1 1 1 1 1 1 1
            1 1 1 1 1 1 1 1 1
            1 1 1 1 1 1 1 1 1
            1 1 1 1 1 1 1 1 1
            1 1 1 1 1 1 1 1 1
        };

        static PROMOTE: [Bitboard; 2] = [SENTE_PROMOTE, SENTE_PROMOTE.flip()];
        static NOPROMOTE: [Bitboard; 2] = [SENTE_NOPROMOTE, SENTE_NOPROMOTE.flip()];

        let mut pawns = self[PieceKind::Pawn] & !self.pieces.promoted & self[self.active];
        pawns &= mask.shift_back(self.active);
        for sq in pawns & NOPROMOTE[self.active] {
            ptry!(r.recv_move(Move::Board {
                from: sq,
                to: unsafe { sq.forward_unchecked(self.active) },
                promoted: false,
            }));
        }
        for sq in pawns & PROMOTE[self.active] {
            ptry!(r.recv_move(Move::Board {
                from: sq,
                to: unsafe { sq.forward_unchecked(self.active) },
                promoted: true,
            }));
        }
        R::Result::output()
    }

    fn lance_moves<R: Receiver>(&self, mask: Bitboard, r: &mut R) -> R::Result {
        (self[PieceKind::Lance] & !self.pieces.promoted & self[self.active])
            .for_each(|sq| lance(self, mask, sq, r))
    }

    fn knight_moves<R: Receiver>(&self, mask: Bitboard, r: &mut R) -> R::Result {
        (self[PieceKind::Knight] & !self.pieces.promoted & self[self.active])
            .for_each(|sq| knight(self, mask, sq, r))
    }

    fn silver_moves<R: Receiver>(&self, mask: Bitboard, r: &mut R) -> R::Result {
        let bb = self[PieceKind::Silver] & !self.pieces.promoted & self[self.active];
        for sq in bb {
            ptry!(silver(self, mask, sq, r));
        }

        let promotion_escape = if self.active == Side::Sente { Rank::C } else { Rank::G };
        for sq in bb & promotion_escape.mask() {
            let back = unsafe { sq.back(self.active).unwrap_unchecked() };
            if let Some(left) = back.left()
                && !self[self.active].contains(left)
                && mask.contains(left)
            {
                ptry!(r.recv_move(Move::Board { from: sq, to: left, promoted: true }));
            }
            if let Some(right) = back.right()
                && !self[self.active].contains(right)
                && mask.contains(right)
            {
                ptry!(r.recv_move(Move::Board { from: sq, to: right, promoted: true }));
            }
        }
        R::Result::output()
    }

    pub(crate) fn gold_move_pieces(&self) -> Bitboard {
        let promoted_to_gold = self[PieceKind::Pawn]
            | self[PieceKind::Lance]
            | self[PieceKind::Knight]
            | self[PieceKind::Silver];
        (promoted_to_gold & self.pieces.promoted) | self[PieceKind::Gold]
    }

    fn gold_moves<R: Receiver>(&self, mask: Bitboard, r: &mut R) -> R::Result {
        (self.gold_move_pieces() & self[self.active]).for_each(|sq| gold(self, mask, sq, r))
    }

    fn unpromoted_bishop_moves<R: Receiver>(&self, mask: Bitboard, r: &mut R) -> R::Result {
        (self[PieceKind::Bishop] & self[self.active] & !self.pieces.promoted)
            .for_each(|sq| bishop::<false, _>(self, mask, sq, r))
    }

    fn promoted_bishop_moves<R: Receiver>(&self, mask: Bitboard, r: &mut R) -> R::Result {
        (self[PieceKind::Bishop] & self[self.active] & self.pieces.promoted)
            .for_each(|sq| bishop::<true, _>(self, mask, sq, r))
    }

    fn unpromoted_rook_moves<R: Receiver>(&self, mask: Bitboard, r: &mut R) -> R::Result {
        (self[PieceKind::Rook] & self[self.active] & !self.pieces.promoted)
            .for_each(|sq| rook::<false, _>(self, mask, sq, r))
    }

    fn promoted_rook_moves<R: Receiver>(&self, mask: Bitboard, r: &mut R) -> R::Result {
        (self[PieceKind::Rook] & self[self.active] & self.pieces.promoted)
            .for_each(|sq| rook::<true, _>(self, mask, sq, r))
    }

    fn king_moves<R: Receiver>(&self, mask: Bitboard, r: &mut R) -> R::Result {
        (self[PieceKind::King] & self[self.active]).for_each(|sq| king(mask, sq, r))
    }
}

fn lance<R: Receiver>(board: &Board, mask: Bitboard, sq: Square, r: &mut R) -> R::Result {
    let bb = lance_moves(sq, board.pieces.all(), board.active) & mask;
    let end_rank = board.active.end_rank();
    ptry!(r.recv(sq, bb & !end_rank.mask(), false));
    r.recv(sq, bb & board.active.promotion_zone(), true)
}

fn knight<R: Receiver>(board: &Board, mask: Bitboard, sq: Square, r: &mut R) -> R::Result {
    ptry!(r.recv(sq, KNIGHT_LUT[board.active][sq] & mask, false));
    r.recv(sq, KNIGHT_PROMOTION_LUT[board.active][sq] & mask, true)
}

fn silver<R: Receiver>(board: &Board, mask: Bitboard, sq: Square, r: &mut R) -> R::Result {
    let bb = SILVER_LUT[board.active][sq] & mask;
    ptry!(r.recv(sq, bb, false));
    r.recv(sq, bb & board.active.promotion_zone(), true)
}

fn gold<R: Receiver>(board: &Board, mask: Bitboard, sq: Square, r: &mut R) -> R::Result {
    r.recv(sq, GOLD_LUT[board.active][sq] & mask, false)
}

fn bishop_rook_finish<const PROMOTED: bool, R: Receiver>(
    board: &Board,
    mask: Bitboard,
    mut bb: Bitboard,
    sq: Square,
    r: &mut R,
) -> R::Result {
    if PROMOTED {
        bb |= KING_LUT[sq];
    }
    bb &= mask;
    ptry!(r.recv(sq, bb, false));
    if PROMOTED {
        return R::Result::output();
    }
    if sq.is_promotion_zone(board.active) {
        r.recv(sq, bb, true)
    } else {
        r.recv(sq, bb & board.active.promotion_zone(), true)
    }
}

fn bishop<const PROMOTED: bool, R: Receiver>(
    board: &Board,
    mask: Bitboard,
    sq: Square,
    r: &mut R,
) -> R::Result {
    bishop_rook_finish::<PROMOTED, _>(board, mask, bishop_moves(sq, board.pieces.all()), sq, r)
}

fn rook<const PROMOTED: bool, R: Receiver>(
    board: &Board,
    mask: Bitboard,
    sq: Square,
    r: &mut R,
) -> R::Result {
    bishop_rook_finish::<PROMOTED, _>(board, mask, rook_moves(sq, board.pieces.all()), sq, r)
}

fn king<R: Receiver>(mask: Bitboard, sq: Square, r: &mut R) -> R::Result {
    r.recv(sq, KING_LUT[sq] & mask, false)
}

const fn compute_knight(side: Side) -> [Bitboard; Square::LEN] {
    let mut moves = [Bitboard::EMPTY; Square::LEN];
    let mut i = 0;
    while (i as usize) < Square::LEN {
        let sq = Square::from_int(i).unwrap();
        if sq.nforward(side, 4).is_none() {
            i += 1;
            continue;
        }

        let forward = sq.nforward(side, 2).unwrap();
        if let Some(left) = forward.left() {
            moves[i as usize].insert(left);
        }
        if let Some(right) = forward.right() {
            moves[i as usize].insert(right);
        }
        i += 1;
    }
    moves
}

const fn compute_knight_promotion(side: Side) -> [Bitboard; Square::LEN] {
    let mut moves = [Bitboard::EMPTY; Square::LEN];
    let mut i = 0;
    while (i as usize) < Square::LEN {
        let sq = Square::from_int(i).unwrap();
        let Some(forward) = sq.nforward(side, 2) else {
            i += 1;
            continue;
        };
        if !forward.is_promotion_zone(side) {
            i += 1;
            continue;
        }

        if let Some(left) = forward.left() {
            moves[i as usize].insert(left);
        }
        if let Some(right) = forward.right() {
            moves[i as usize].insert(right);
        }
        i += 1;
    }
    moves
}

const fn compute_silver(side: Side) -> [Bitboard; Square::LEN] {
    let mut moves = [Bitboard::EMPTY; Square::LEN];
    let mut index = 0;
    while (index as usize) < Square::LEN {
        let sq = Square::from_int(index).unwrap();
        let mut bb = Bitboard::EMPTY;
        if let Some(forward) = sq.rank().forward(side) {
            bb.insert(Square::new(sq.file(), forward));
            if let Some(left) = sq.file().left() {
                bb.insert(Square::new(left, forward));
            }
            if let Some(right) = sq.file().right() {
                bb.insert(Square::new(right, forward));
            }
        }
        if let Some(back) = sq.rank().back(side) {
            if let Some(left) = sq.file().left() {
                bb.insert(Square::new(left, back));
            }
            if let Some(right) = sq.file().right() {
                bb.insert(Square::new(right, back));
            }
        }
        moves[index as usize] = bb;
        index += 1;
    }
    moves
}

const fn compute_gold(side: Side) -> [Bitboard; Square::LEN] {
    let mut moves = [Bitboard::EMPTY; Square::LEN];
    let mut index = 0;
    while (index as usize) < Square::LEN {
        let sq = Square::from_int(index).unwrap();
        let mut bb = Bitboard::EMPTY;
        if let Some(left) = sq.file().left() {
            bb.insert(Square::new(left, sq.rank()));
        }
        if let Some(right) = sq.file().right() {
            bb.insert(Square::new(right, sq.rank()));
        }
        if let Some(forward) = sq.rank().forward(side) {
            bb.insert(Square::new(sq.file(), forward));
            if let Some(left) = sq.file().left() {
                bb.insert(Square::new(left, forward));
            }
            if let Some(right) = sq.file().right() {
                bb.insert(Square::new(right, forward));
            }
        }
        if let Some(back) = sq.rank().back(side) {
            bb.insert(Square::new(sq.file(), back));
        }
        moves[index as usize] = bb;
        index += 1;
    }
    moves
}

const fn compute_king() -> [Bitboard; Square::LEN] {
    let mut moves = [Bitboard::EMPTY; Square::LEN];
    let mut index = 0;
    while (index as usize) < Square::LEN {
        let sq = Square::from_int(index).unwrap();
        let mut bb = Bitboard::EMPTY;
        if let Some(left) = sq.file().left() {
            bb.insert(Square::new(left, sq.rank()));
        }
        if let Some(right) = sq.file().right() {
            bb.insert(Square::new(right, sq.rank()));
        }
        if let Some(up) = sq.rank().up() {
            bb.insert(Square::new(sq.file(), up));
            if let Some(left) = sq.file().left() {
                bb.insert(Square::new(left, up));
            }
            if let Some(right) = sq.file().right() {
                bb.insert(Square::new(right, up));
            }
        }
        if let Some(down) = sq.rank().down() {
            bb.insert(Square::new(sq.file(), down));
            if let Some(left) = sq.file().left() {
                bb.insert(Square::new(left, down));
            }
            if let Some(right) = sq.file().right() {
                bb.insert(Square::new(right, down));
            }
        }
        moves[index as usize] = bb;
        index += 1;
    }
    moves
}

#[test]
fn test_is_legal() {
    use std::ops::ControlFlow;
    let board = Board::start_pos();
    for from in Square::ALL {
        for to in Square::ALL {
            for promote in [false, true] {
                let mov = Move::Board { from, to, promoted: promote };
                let is_legal = board.is_legal(mov);
                let has_legal = board
                    .legal_moves(|m| {
                        if m == mov { ControlFlow::Break(()) } else { ControlFlow::Continue(()) }
                    })
                    .is_break();
                assert_eq!(is_legal, has_legal, "{mov}");
            }
        }
    }
}
