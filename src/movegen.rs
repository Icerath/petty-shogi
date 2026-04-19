use crate::{Action, Bitboard, Board, Piece, PieceKind, Side, Square, bitboard};

pub trait Receiver {
    fn recv(&mut self, action: Action);
}

impl Receiver for Vec<Action> {
    fn recv(&mut self, action: Action) {
        self.push(action);
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
            && piece.kind() == PieceKind::Pawn
            && (self[PieceKind::King] & self[!self.active])
                .contains(to.forward(self.active).unwrap())
        {
            let mut board = self.clone();
            board.play(action);
            let mut any = false;
            board.legal_moves(&mut |_| any = true);
            if !any {
                return false;
            }
        }

        let mut board = self.clone();
        board.play(action);
        let Some(king_square) = (board[PieceKind::King] & board[self.active]).bitscan() else {
            return false;
        };
        !board.is_square_attacked(king_square, self.active)
    }

    #[must_use]
    pub fn is_square_attacked(&self, sq: Square, side: Side) -> bool {
        let occupancy = self.pieces.all();
        macro_rules! check {
            ($($bb:expr),* $(,)?) => {
                $(!($bb & self[!side]).is_empty())||*
            };
        }

        check! {
            (self[PieceKind::Pawn] & !self.pieces.promoted) & sq.forward(side).map_or(Bitboard::EMPTY, Square::mask),
            slide(sq, occupancy, 0, side.forward()) & self[PieceKind::Lance] & !self.pieces.promoted,
            KNIGHT_LUT[side as usize][sq as usize] & self[PieceKind::Knight] & !self.pieces.promoted,
            SILVER_LUT[side as usize][sq as usize] & self[PieceKind::Silver] & !self.pieces.promoted,
            GOLD_LUT[side as usize][sq as usize] & self.gold_move_pieces(),
            bishop_bb(occupancy, sq) & self[PieceKind::Bishop],
            rook_bb(occupancy, sq) & self[PieceKind::Rook],
            KING_LUT[sq as usize] & (self[PieceKind::King] | ((self[PieceKind::Bishop] | self[PieceKind::Rook]) & self.pieces.promoted)),
        }
    }

    pub fn legal_moves<'a, R: Receiver>(&self, r: &'a mut R) -> &'a mut R {
        self.pseudolegal_moves(&mut |mov| {
            if self.is_legal(mov) {
                r.recv(mov)
            }
        });
        r
    }

    pub fn pseudolegal_moves<'a, R: Receiver>(&self, r: &'a mut R) -> &'a mut R {
        self.pawn_moves(r);
        self.lance_moves(r);
        self.knight_moves(r);
        self.silver_moves(r);
        self.gold_moves(r);
        self.bishop_moves(r);
        self.rook_moves(r);
        self.king_moves(r);
        self.drop_moves(r);
        r
    }

    fn drop_moves(&self, r: &mut impl Receiver) {
        let empty_squares = !self.pieces.all();

        macro_rules! drop {
            ($piece:expr, $to:expr) => {
                r.recv(Action::Drop { piece: Piece::new(self.active, $piece, false), to: $to })
            };
        }

        // pawns
        if self.hands[self.active as usize][PieceKind::Pawn as usize] > 0 {
            let mut busy_files = Bitboard::EMPTY;
            (self[PieceKind::Pawn] & !self.pieces.promoted & self[self.active]).for_each(|sq| {
                busy_files |= sq.file().mask();
            });
            (empty_squares & !self.active.end_rank().mask() & !busy_files)
                .for_each(|sq| drop!(PieceKind::Pawn, sq));
        }
        // lances
        if self.hands[self.active as usize][PieceKind::Lance as usize] > 0 {
            (empty_squares & !self.active.end_rank().mask())
                .for_each(|sq| drop!(PieceKind::Lance, sq));
        }
        // knights
        if self.hands[self.active as usize][PieceKind::Lance as usize] > 0 {
            (empty_squares & !self.active.promotion_zone().shift_forward(self.active))
                .for_each(|sq| drop!(PieceKind::Knight, sq));
        }
        // rest
        for &piece in &PieceKind::ALL[PieceKind::Silver as usize..PieceKind::King as usize] {
            if self.hands[self.active as usize][piece as usize] == 0 {
                continue;
            }
            empty_squares.for_each(|sq| drop!(piece, sq));
        }
    }

    fn pawn_moves(&self, r: &mut impl Receiver) {
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
        const GOTE_PROMOTE: Bitboard = SENTE_PROMOTE.flip();
        const GOTE_NOPROMOTE: Bitboard = SENTE_NOPROMOTE.flip();

        let pawns = self[PieceKind::Pawn] & !self.pieces.promoted & self[self.active];

        macro_rules! moves {
            ($side:expr, $($mask:ident: $promote:literal),*) => {{
                let blocked_pawns = self[$side].shift_forward($side);
                $(
                    (pawns & $mask & !blocked_pawns).for_each(|sq| {
                        r.recv(Action::Move {
                            from: sq,
                            to: unsafe { sq.forward($side).unwrap_unchecked() },
                            promoted: $promote,
                        })
                    });
                )*
            }};
        }
        match self.active {
            Side::Sente => moves!(Side::Sente, SENTE_NOPROMOTE: false, SENTE_PROMOTE: true),
            Side::Gote => moves!(Side::Gote, GOTE_NOPROMOTE: false, GOTE_PROMOTE: true),
        }
    }

    fn lance_moves(&self, r: &mut impl Receiver) {
        (self[PieceKind::Lance] & !self.pieces.promoted & self[self.active])
            .for_each(|sq| lance(self, sq, r));
    }

    fn knight_moves(&self, r: &mut impl Receiver) {
        (self[PieceKind::Knight] & !self.pieces.promoted & self[self.active])
            .for_each(|sq| knight(self, sq, r));
    }

    fn silver_moves(&self, r: &mut impl Receiver) {
        (self[PieceKind::Silver] & !self.pieces.promoted & self[self.active])
            .for_each(|sq| silver(self, sq, r));
    }

    fn gold_move_pieces(&self) -> Bitboard {
        let promoted_to_gold = self[PieceKind::Pawn]
            | self[PieceKind::Lance]
            | self[PieceKind::Knight]
            | self[PieceKind::Silver];
        (promoted_to_gold & self.pieces.promoted) | self[PieceKind::Gold]
    }

    fn gold_moves(&self, r: &mut impl Receiver) {
        (self.gold_move_pieces() & self[self.active]).for_each(|sq| gold(self, sq, r));
    }

    fn bishop_moves(&self, r: &mut impl Receiver) {
        (self[PieceKind::Bishop] & self[self.active] & !self.pieces.promoted)
            .for_each(|sq| bishop::<false>(self, sq, r));
        (self[PieceKind::Bishop] & self[self.active] & self.pieces.promoted)
            .for_each(|sq| bishop::<true>(self, sq, r));
    }

    #[inline(never)]
    fn rook_moves(&self, r: &mut impl Receiver) {
        (self[PieceKind::Rook] & self[self.active] & !self.pieces.promoted)
            .for_each(|sq| rook::<false>(self, sq, r));
        (self[PieceKind::Rook] & self[self.active] & self.pieces.promoted)
            .for_each(|sq| rook::<true>(self, sq, r));
    }

    fn king_moves(&self, r: &mut impl Receiver) {
        (self[PieceKind::King] & self[self.active]).for_each(|sq| king(self, sq, r));
    }
}

fn lance(board: &Board, sq: Square, r: &mut impl Receiver) {
    let bb = slide(sq, board.pieces.all(), 0, board.active.forward()) & !board[board.active];
    let end_rank = board.active.end_rank();
    (bb & !end_rank.mask()).for_each(|to| r.recv(Action::Move { from: sq, to, promoted: false }));
    (bb & board.active.promotion_zone())
        .for_each(|to| r.recv(Action::Move { from: sq, to, promoted: true }));
}

fn knight(board: &Board, sq: Square, r: &mut impl Receiver) {
    (KNIGHT_LUT[board.active as usize][sq as usize] & !board[board.active])
        .for_each(|to| r.recv(Action::Move { from: sq, to, promoted: false }));
    (KNIGHT_PROMOTION_LUT[board.active as usize][sq as usize] & !board[board.active])
        .for_each(|to| r.recv(Action::Move { from: sq, to, promoted: true }));
}

fn silver(board: &Board, sq: Square, r: &mut impl Receiver) {
    (SILVER_LUT[board.active as usize][sq as usize] & !board[board.active])
        .for_each(|to| r.recv(Action::Move { from: sq, to, promoted: false }));
    (SILVER_PROMOTION_LUT[board.active as usize][sq as usize] & !board[board.active])
        .for_each(|to| r.recv(Action::Move { from: sq, to, promoted: true }));
}

fn gold(board: &Board, sq: Square, r: &mut impl Receiver) {
    (GOLD_LUT[board.active as usize][sq as usize] & !board[board.active])
        .for_each(|to| r.recv(Action::Move { from: sq, to, promoted: false }));
}

fn bishop<const PROMOTED: bool>(board: &Board, sq: Square, r: &mut impl Receiver) {
    let mut bb = bishop_bb(board.pieces.all(), sq) & !board[board.active];
    if PROMOTED {
        bb |= KING_LUT[sq as usize];
    }

    bb.for_each(|to| r.recv(Action::Move { from: sq, to, promoted: false }));
    if !PROMOTED {
        (bb & board.active.promotion_zone())
            .for_each(|to| r.recv(Action::Move { from: sq, to, promoted: true }));
    }
}

fn bishop_bb(occupancy: Bitboard, sq: Square) -> Bitboard {
    let mut bb = Bitboard::EMPTY;
    for (h, v) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
        bb |= slide(sq, occupancy, h, v)
    }
    bb
}

fn rook<const PROMOTED: bool>(board: &Board, sq: Square, r: &mut impl Receiver) {
    let mut bb = rook_bb(board.pieces.all(), sq) & !board[board.active];
    if PROMOTED {
        bb |= KING_LUT[sq as usize];
    }
    bb.for_each(|to| r.recv(Action::Move { from: sq, to, promoted: false }));
    if !PROMOTED {
        (bb & board.active.promotion_zone())
            .for_each(|to| r.recv(Action::Move { from: sq, to, promoted: true }));
    }
}

fn rook_bb(occupancy: Bitboard, sq: Square) -> Bitboard {
    let mut bb = Bitboard::EMPTY;
    for (h, v) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
        bb |= slide(sq, occupancy, h, v)
    }
    bb
}

fn king(board: &Board, sq: Square, r: &mut impl Receiver) {
    (KING_LUT[sq as usize] & !board[board.active]).for_each(|to| {
        r.recv(Action::Move { from: sq, to, promoted: false });
    });
}

static KNIGHT_LUT: [[Bitboard; 81]; 2] = [compute_knight(Side::Sente), compute_knight(Side::Gote)];
static KNIGHT_PROMOTION_LUT: [[Bitboard; 81]; 2] =
    [compute_knight_promotion(Side::Sente), compute_knight_promotion(Side::Gote)];

static SILVER_LUT: [[Bitboard; 81]; 2] = [compute_silver(Side::Sente), compute_silver(Side::Gote)];
static SILVER_PROMOTION_LUT: [[Bitboard; 81]; 2] = [
    promotion_moves(compute_silver(Side::Sente), Side::Sente),
    promotion_moves(compute_silver(Side::Gote), Side::Gote),
];

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

const fn promotion_moves(mut moves: [Bitboard; 81], side: Side) -> [Bitboard; 81] {
    let mut i = 0;
    while i < 81 {
        moves[i] = moves[i].bitand(side.promotion_zone());
        i += 1;
    }
    moves
}

#[must_use]
fn slide(mut from: Square, occupancy: Bitboard, h: i8, v: i8) -> Bitboard {
    let mut output = Bitboard::EMPTY;
    while let Some(sq) = from.offset_file_rank(h, v) {
        from = sq;
        output.insert(sq);
        if occupancy.contains(sq) {
            break;
        }
    }
    output
}
