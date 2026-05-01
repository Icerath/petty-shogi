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
        let mask = !self[self.active] & mask;
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
                bishop_moves,
                rook_moves,
                drop_moves,
            );
        }
        self.king_moves(mask, r)
    }

    fn drop_moves<R: Receiver>(&self, mask: Bitboard, r: &mut R) -> R::Result {
        if let Some(true) = R::PROMOTE_FILTER {
            return R::Result::output();
        }
        let empty_squares = mask & !self[!self.active];

        if empty_squares.is_empty() {
            return R::Result::output();
        }

        macro_rules! drop {
            ($piece:expr, $to:expr) => {
                r.recv(Move::Drop { piece: $piece, to: $to })
            };
        }

        // pawns
        if self.hands[self.active][PieceKind::Pawn] > 0 {
            let mut busy_files = Bitboard::EMPTY;
            (self[PieceKind::Pawn] & !self.pieces.promoted & self[self.active]).for_each(|sq| {
                busy_files |= sq.file().mask();
            });
            ptry!(
                (empty_squares & !self.active.end_rank().mask() & !busy_files)
                    .for_each(|sq| drop!(PieceKind::Pawn, sq))
            );
        }
        // lances
        if self.hands[self.active][PieceKind::Lance] > 0 {
            ptry!(
                (empty_squares & !self.active.end_rank().mask())
                    .for_each(|sq| drop!(PieceKind::Lance, sq))
            );
        }
        // knights
        if self.hands[self.active][PieceKind::Knight] > 0 {
            ptry!(
                (empty_squares & !self.active.promotion_zone().shift_forward(self.active))
                    .for_each(|sq| drop!(PieceKind::Knight, sq))
            );
        }
        // rest
        for &piece in &PieceKind::ALL[PieceKind::Silver as usize..PieceKind::King as usize] {
            if self.hands[self.active][piece] == 0 {
                continue;
            }
            ptry!(empty_squares.for_each(|sq| drop!(piece, sq)));
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
        if let Some(false) | None = R::PROMOTE_FILTER {
            ptry!((pawns & NOPROMOTE[self.active]).for_each(|sq| {
                r.recv(Move::Board {
                    from: sq,
                    to: unsafe { sq.forward_unchecked(self.active) },
                    promoted: false,
                })
            }));
        }
        if let Some(true) | None = R::PROMOTE_FILTER {
            ptry!((pawns & PROMOTE[self.active]).for_each(|sq| {
                r.recv(Move::Board {
                    from: sq,
                    to: unsafe { sq.forward_unchecked(self.active) },
                    promoted: true,
                })
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
        ptry!(bb.for_each(|sq| silver(self, mask, sq, r)));

        if let Some(false) = R::PROMOTE_FILTER {
            return R::Result::output();
        }

        let promotion_escape = if self.active == Side::Sente { Rank::C } else { Rank::G };
        (bb & promotion_escape.mask()).for_each(|sq| {
            let back = unsafe { sq.back(self.active).unwrap_unchecked() };
            if let Some(left) = back.left()
                && !self[self.active].contains(left)
            {
                ptry!(r.recv(Move::Board { from: sq, to: left, promoted: true }));
            }
            if let Some(right) = back.right()
                && !self[self.active].contains(right)
            {
                ptry!(r.recv(Move::Board { from: sq, to: right, promoted: true }));
            }
            R::Result::output()
        })
    }

    pub(crate) fn gold_move_pieces(&self) -> Bitboard {
        let promoted_to_gold = self[PieceKind::Pawn]
            | self[PieceKind::Lance]
            | self[PieceKind::Knight]
            | self[PieceKind::Silver];
        (promoted_to_gold & self.pieces.promoted) | self[PieceKind::Gold]
    }

    fn gold_moves<R: Receiver>(&self, mask: Bitboard, r: &mut R) -> R::Result {
        if let Some(true) = R::PROMOTE_FILTER {
            return R::Result::output();
        }
        (self.gold_move_pieces() & self[self.active]).for_each(|sq| gold(self, mask, sq, r))
    }

    fn bishop_moves<R: Receiver>(&self, mask: Bitboard, r: &mut R) -> R::Result {
        ptry!(
            (self[PieceKind::Bishop] & self[self.active] & !self.pieces.promoted)
                .for_each(|sq| bishop::<false, _>(self, mask, sq, r))
        );
        (self[PieceKind::Bishop] & self[self.active] & self.pieces.promoted)
            .for_each(|sq| bishop::<true, _>(self, mask, sq, r))
    }

    fn rook_moves<R: Receiver>(&self, mask: Bitboard, r: &mut R) -> R::Result {
        ptry!(
            (self[PieceKind::Rook] & self[self.active] & !self.pieces.promoted)
                .for_each(|sq| rook::<false, _>(self, mask, sq, r))
        );
        (self[PieceKind::Rook] & self[self.active] & self.pieces.promoted)
            .for_each(|sq| rook::<true, _>(self, mask, sq, r))
    }

    fn king_moves<R: Receiver>(&self, mask: Bitboard, r: &mut R) -> R::Result {
        if let Some(true) = R::PROMOTE_FILTER {
            return R::Result::output();
        }
        (self[PieceKind::King] & self[self.active]).for_each(|sq| king(mask, sq, r))
    }
}

fn lance<R: Receiver>(board: &Board, mask: Bitboard, sq: Square, r: &mut R) -> R::Result {
    let bb = lance_moves(sq, board.pieces.all(), board.active) & mask;
    let end_rank = board.active.end_rank();
    if let Some(false) | None = R::PROMOTE_FILTER {
        ptry!((bb & !end_rank.mask()).for_each(|to| r.recv(Move::Board {
            from: sq,
            to,
            promoted: false
        })));
    }
    if let Some(true) | None = R::PROMOTE_FILTER {
        ptry!((bb & board.active.promotion_zone()).for_each(|to| r.recv(Move::Board {
            from: sq,
            to,
            promoted: true
        })));
    }
    R::Result::output()
}

fn knight<R: Receiver>(board: &Board, mask: Bitboard, sq: Square, r: &mut R) -> R::Result {
    if let Some(false) | None = R::PROMOTE_FILTER {
        ptry!((KNIGHT_LUT[board.active][sq] & mask).for_each(|to| r.recv(Move::Board {
            from: sq,
            to,
            promoted: false
        })));
    }
    if let Some(true) | None = R::PROMOTE_FILTER {
        ptry!((KNIGHT_PROMOTION_LUT[board.active][sq] & mask).for_each(|to| r.recv(Move::Board {
            from: sq,
            to,
            promoted: true
        })));
    }
    R::Result::output()
}

fn silver<R: Receiver>(board: &Board, mask: Bitboard, sq: Square, r: &mut R) -> R::Result {
    let bb = SILVER_LUT[board.active][sq] & mask;
    if let Some(false) | None = R::PROMOTE_FILTER {
        ptry!(bb.for_each(|to| r.recv(Move::Board { from: sq, to, promoted: false })));
    }
    if let Some(true) | None = R::PROMOTE_FILTER {
        ptry!((bb & board.active.promotion_zone()).for_each(|to| r.recv(Move::Board {
            from: sq,
            to,
            promoted: true
        })));
    }
    R::Result::output()
}

fn gold<R: Receiver>(board: &Board, mask: Bitboard, sq: Square, r: &mut R) -> R::Result {
    (GOLD_LUT[board.active][sq] & mask)
        .for_each(|to| r.recv(Move::Board { from: sq, to, promoted: false }))
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
    if let Some(false) | None = R::PROMOTE_FILTER {
        ptry!(bb.for_each(|to| r.recv(Move::Board { from: sq, to, promoted: false })));
    }
    if PROMOTED {
        return R::Result::output();
    }
    if sq.is_promotion_zone(board.active) {
        if let Some(true) | None = R::PROMOTE_FILTER {
            bb.for_each(|to| r.recv(Move::Board { from: sq, to, promoted: true }))
        } else {
            R::Result::output()
        }
    } else {
        if let Some(false) | None = R::PROMOTE_FILTER {
            (bb & board.active.promotion_zone())
                .for_each(|to| r.recv(Move::Board { from: sq, to, promoted: true }))
        } else {
            R::Result::output()
        }
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
    (KING_LUT[sq] & mask).for_each(|to| r.recv(Move::Board { from: sq, to, promoted: false }))
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
