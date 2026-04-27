pub mod magic;

use magic::{bishop_moves, lance_moves, rook_moves};

use crate::{Action, Bitboard, Board, PieceKind, Rank, Side, Square, bitboard};
pub trait Receiver {
    fn recv(&mut self, action: Action);
}

impl Receiver for Vec<Action> {
    fn recv(&mut self, action: Action) {
        self.push(action);
    }
}

impl Receiver for &mut Vec<Action> {
    fn recv(&mut self, action: Action) {
        (*self).recv(action)
    }
}

impl Receiver for u64 {
    fn recv(&mut self, _: Action) {
        *self += 1;
    }
}

impl<F: FnMut(Action)> Receiver for F {
    fn recv(&mut self, action: Action) {
        (*self)(action)
    }
}

impl Board {
    pub fn is_legal(&self, action: Action) -> bool {
        // check for pawn drop mate
        if let Action::Drop { piece, to } = action
            && piece == PieceKind::Pawn
            && (self[PieceKind::King] & self[!self.active])
                .contains(to.forward(self.active).unwrap())
        {
            let mut board = self.clone();
            board.play(action);
            let mut any = false;
            _ = board.legal_moves(|_| any = true);
            if !any {
                return false;
            }
        }

        let mut board = self.clone();
        board.play(action);
        let Some(king_square) = (board[PieceKind::King] & board[self.active]).bitscan() else {
            return true;
        };
        board.gen_attackers(king_square, self.active).is_empty()
    }

    #[must_use]
    pub fn is_square_attacked(&self, sq: Square, side: Side) -> bool {
        !self.gen_attackers(sq, side).is_empty()
    }

    #[must_use]
    #[inline(always)]
    pub fn gen_attackers(&self, sq: Square, side: Side) -> Bitboard {
        let occupancy = self.pieces.all();
        macro_rules! attackers {
            ($($bb:expr),* $(,)?) => {
                let mut attackers = Bitboard::EMPTY;
                $(attackers |= ($bb & self[!side]);)*
                attackers
            };
        }
        attackers! {
            (self[PieceKind::Pawn] & !self.pieces.promoted) & sq.forward(side).map_or(Bitboard::EMPTY, Square::mask),
            lance_moves(sq, occupancy, side) & self[PieceKind::Lance] & !self.pieces.promoted,
            KNIGHT_LUT[side][sq] & self[PieceKind::Knight] & !self.pieces.promoted,
            SILVER_LUT[side][sq] & self[PieceKind::Silver] & !self.pieces.promoted,
            GOLD_LUT[side][sq] & self.gold_move_pieces(),
            bishop_moves(sq, occupancy) & self[PieceKind::Bishop],
            rook_moves(sq, occupancy) & self[PieceKind::Rook],
            KING_LUT[sq] & (self[PieceKind::King] | ((self[PieceKind::Bishop] | self[PieceKind::Rook]) & self.pieces.promoted)),
        }
    }

    #[must_use]
    pub fn is_check(&self) -> bool {
        let Some(king_square) = (self[PieceKind::King] & self[self.active]).bitscan() else {
            return false;
        };
        self.is_square_attacked(king_square, self.active)
    }

    pub fn legal_moves<R: Receiver>(&self, mut r: R) -> R {
        _ = self.pseudolegal_moves(|mov| {
            if self.is_legal(mov) {
                r.recv(mov)
            }
        });
        r
    }

    pub fn pseudolegal_moves<R: Receiver>(&self, mut r: R) -> R {
        let checkers = match (self[PieceKind::King] & self[self.active]).bitscan() {
            Some(king_square) => self.gen_attackers(king_square, self.active),
            None => Bitboard::EMPTY,
        };
        let mask = !self[self.active];
        if checkers.count() < 2 {
            self.drop_moves(mask, &mut r);
            self.pawn_moves(mask, &mut r);
            self.lance_moves(mask, &mut r);
            self.knight_moves(mask, &mut r);
            self.silver_moves(mask, &mut r);
            self.gold_moves(mask, &mut r);
            self.bishop_moves(mask, &mut r);
            self.rook_moves(mask, &mut r);
        }
        self.king_moves(mask, &mut r);
        r
    }

    fn drop_moves(&self, mask: Bitboard, r: &mut impl Receiver) {
        let empty_squares = mask & !self[!self.active];

        macro_rules! drop {
            ($piece:expr, $to:expr) => {
                r.recv(Action::Drop { piece: $piece, to: $to })
            };
        }

        // pawns
        if self.hands[self.active][PieceKind::Pawn] > 0 {
            let mut busy_files = Bitboard::EMPTY;
            (self[PieceKind::Pawn] & !self.pieces.promoted & self[self.active]).for_each(|sq| {
                busy_files |= sq.file().mask();
            });
            (empty_squares & !self.active.end_rank().mask() & !busy_files)
                .for_each(|sq| drop!(PieceKind::Pawn, sq));
        }
        // lances
        if self.hands[self.active][PieceKind::Lance] > 0 {
            (empty_squares & !self.active.end_rank().mask())
                .for_each(|sq| drop!(PieceKind::Lance, sq));
        }
        // knights
        if self.hands[self.active][PieceKind::Knight] > 0 {
            (empty_squares & !self.active.promotion_zone().shift_forward(self.active))
                .for_each(|sq| drop!(PieceKind::Knight, sq));
        }
        // rest
        for &piece in &PieceKind::ALL[PieceKind::Silver as usize..PieceKind::King as usize] {
            if self.hands[self.active][piece] == 0 {
                continue;
            }
            empty_squares.for_each(|sq| drop!(piece, sq));
        }
    }

    fn pawn_moves(&self, mask: Bitboard, r: &mut impl Receiver) {
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

        const PROMOTE: [Bitboard; 2] = [SENTE_PROMOTE, SENTE_PROMOTE.flip()];
        const NOPROMOTE: [Bitboard; 2] = [SENTE_NOPROMOTE, SENTE_NOPROMOTE.flip()];

        let mut pawns = self[PieceKind::Pawn] & !self.pieces.promoted & self[self.active];
        pawns &= mask.shift_back(self.active);
        (pawns & NOPROMOTE[self.active]).for_each(|sq| {
            r.recv(Action::Move {
                from: sq,
                to: unsafe { sq.forward_unchecked(self.active) },
                promoted: false,
            })
        });
        (pawns & PROMOTE[self.active]).for_each(|sq| {
            r.recv(Action::Move {
                from: sq,
                to: unsafe { sq.forward_unchecked(self.active) },
                promoted: true,
            })
        });
    }

    fn lance_moves(&self, mask: Bitboard, r: &mut impl Receiver) {
        (self[PieceKind::Lance] & !self.pieces.promoted & self[self.active])
            .for_each(|sq| lance(self, mask, sq, r));
    }

    fn knight_moves(&self, mask: Bitboard, r: &mut impl Receiver) {
        (self[PieceKind::Knight] & !self.pieces.promoted & self[self.active])
            .for_each(|sq| knight(self, mask, sq, r));
    }

    fn silver_moves(&self, mask: Bitboard, r: &mut impl Receiver) {
        let bb = self[PieceKind::Silver] & !self.pieces.promoted & self[self.active];
        bb.for_each(|sq| silver(self, mask, sq, r));
        let promotion_escape = if self.active == Side::Sente { Rank::C } else { Rank::G };
        (bb & promotion_escape.mask()).for_each(|sq| {
            let back = unsafe { sq.back(self.active).unwrap_unchecked() };
            if let Some(left) = back.left()
                && !self[self.active].contains(left)
            {
                r.recv(Action::Move { from: sq, to: left, promoted: true });
            }
            if let Some(right) = back.right()
                && !self[self.active].contains(right)
            {
                r.recv(Action::Move { from: sq, to: right, promoted: true });
            }
        });
    }

    fn gold_move_pieces(&self) -> Bitboard {
        let promoted_to_gold = self[PieceKind::Pawn]
            | self[PieceKind::Lance]
            | self[PieceKind::Knight]
            | self[PieceKind::Silver];
        (promoted_to_gold & self.pieces.promoted) | self[PieceKind::Gold]
    }

    fn gold_moves(&self, mask: Bitboard, r: &mut impl Receiver) {
        let bb = self.gold_move_pieces() & self[self.active];
        bb.for_each(|sq| gold(self, mask, sq, r));
    }

    fn bishop_moves(&self, mask: Bitboard, r: &mut impl Receiver) {
        (self[PieceKind::Bishop] & self[self.active] & !self.pieces.promoted)
            .for_each(|sq| bishop::<false>(self, mask, sq, r));
        (self[PieceKind::Bishop] & self[self.active] & self.pieces.promoted)
            .for_each(|sq| bishop::<true>(self, mask, sq, r));
    }

    #[inline(never)]
    fn rook_moves(&self, mask: Bitboard, r: &mut impl Receiver) {
        (self[PieceKind::Rook] & self[self.active] & !self.pieces.promoted)
            .for_each(|sq| rook::<false>(self, mask, sq, r));
        (self[PieceKind::Rook] & self[self.active] & self.pieces.promoted)
            .for_each(|sq| rook::<true>(self, mask, sq, r));
    }

    fn king_moves(&self, mask: Bitboard, r: &mut impl Receiver) {
        (self[PieceKind::King] & self[self.active]).for_each(|sq| king(mask, sq, r));
    }
}

fn lance(board: &Board, mask: Bitboard, sq: Square, r: &mut impl Receiver) {
    let bb = lance_moves(sq, board.pieces.all(), board.active) & mask;
    let end_rank = board.active.end_rank();
    (bb & !end_rank.mask()).for_each(|to| r.recv(Action::Move { from: sq, to, promoted: false }));
    (bb & board.active.promotion_zone())
        .for_each(|to| r.recv(Action::Move { from: sq, to, promoted: true }));
}

fn knight(board: &Board, mask: Bitboard, sq: Square, r: &mut impl Receiver) {
    (KNIGHT_LUT[board.active][sq] & mask)
        .for_each(|to| r.recv(Action::Move { from: sq, to, promoted: false }));
    (KNIGHT_PROMOTION_LUT[board.active][sq] & mask)
        .for_each(|to| r.recv(Action::Move { from: sq, to, promoted: true }));
}

fn silver(board: &Board, mask: Bitboard, sq: Square, r: &mut impl Receiver) {
    let bb = SILVER_LUT[board.active][sq] & mask;
    bb.for_each(|to| r.recv(Action::Move { from: sq, to, promoted: false }));
    (bb & board.active.promotion_zone())
        .for_each(|to| r.recv(Action::Move { from: sq, to, promoted: true }));
}

fn gold(board: &Board, mask: Bitboard, sq: Square, r: &mut impl Receiver) {
    (GOLD_LUT[board.active][sq] & mask)
        .for_each(|to| r.recv(Action::Move { from: sq, to, promoted: false }));
}

fn bishop_rook_finish<const PROMOTED: bool>(
    board: &Board,
    mask: Bitboard,
    mut bb: Bitboard,
    sq: Square,
    r: &mut impl Receiver,
) {
    if PROMOTED {
        bb |= KING_LUT[sq];
    }
    bb &= mask;
    bb.for_each(|to| r.recv(Action::Move { from: sq, to, promoted: false }));
    if !PROMOTED {
        if sq.is_promotion_zone(board.active) {
            bb.for_each(|to| r.recv(Action::Move { from: sq, to, promoted: true }));
        } else {
            (bb & board.active.promotion_zone())
                .for_each(|to| r.recv(Action::Move { from: sq, to, promoted: true }));
        }
    }
}

fn bishop<const PROMOTED: bool>(board: &Board, mask: Bitboard, sq: Square, r: &mut impl Receiver) {
    bishop_rook_finish::<PROMOTED>(board, mask, bishop_moves(sq, board.pieces.all()), sq, r);
}

fn rook<const PROMOTED: bool>(board: &Board, mask: Bitboard, sq: Square, r: &mut impl Receiver) {
    bishop_rook_finish::<PROMOTED>(board, mask, rook_moves(sq, board.pieces.all()), sq, r);
}

fn king(mask: Bitboard, sq: Square, r: &mut impl Receiver) {
    (KING_LUT[sq] & mask).for_each(|to| {
        r.recv(Action::Move { from: sq, to, promoted: false });
    });
}

static KNIGHT_LUT: [[Bitboard; 81]; 2] = [compute_knight(Side::Sente), compute_knight(Side::Gote)];
static KNIGHT_PROMOTION_LUT: [[Bitboard; 81]; 2] =
    [compute_knight_promotion(Side::Sente), compute_knight_promotion(Side::Gote)];

static SILVER_LUT: [[Bitboard; 81]; 2] = [compute_silver(Side::Sente), compute_silver(Side::Gote)];

static GOLD_LUT: [[Bitboard; 81]; 2] = [compute_gold(Side::Sente), compute_gold(Side::Gote)];
static KING_LUT: [Bitboard; 81] = compute_king();

const fn compute_knight(side: Side) -> [Bitboard; 81] {
    let mut moves = [Bitboard::EMPTY; 81];
    let mut i = 0;
    while i < 81 {
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

const fn compute_knight_promotion(side: Side) -> [Bitboard; 81] {
    let mut moves = [Bitboard::EMPTY; 81];
    let mut i = 0;
    while i < 81 {
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

const fn compute_silver(side: Side) -> [Bitboard; 81] {
    let mut moves = [Bitboard::EMPTY; 81];
    let mut index = 0;
    while index < 81 {
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

const fn compute_gold(side: Side) -> [Bitboard; 81] {
    let mut moves = [Bitboard::EMPTY; 81];
    let mut index = 0;
    while index < 81 {
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

const fn compute_king() -> [Bitboard; 81] {
    let mut moves = [Bitboard::EMPTY; 81];
    let mut index = 0;
    while index < 81 {
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
