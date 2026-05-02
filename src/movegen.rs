pub mod magic;
mod pseudolegal;
mod receiver;

use std::ops::ControlFlow;

use magic::{bishop_moves, lance_moves, rook_moves};
use pseudolegal::{GOLD_LUT, KING_LUT, KNIGHT_LUT, SILVER_LUT};
pub use receiver::{FilterPromote, Receiver};

use crate::{Bitboard, Board, Move, PieceKind, Side, Square};

impl Board {
    #[must_use]
    pub fn has_legal_move(&self, mov: Move) -> bool {
        self.legal_moves(|legal| {
            if legal == mov { ControlFlow::Break(()) } else { ControlFlow::Continue(()) }
        })
        .is_break()
    }

    fn any_legal_move(&self) -> bool {
        self.legal_moves(|_| ControlFlow::Break(())).is_break()
    }

    #[expect(clippy::missing_panics_doc)]
    #[must_use]
    pub fn is_legal(&self, mov: Move) -> bool {
        let mut board = self.clone();
        board.play(mov);

        // check for pawn drop mate
        if let Move::Drop { piece, to } = mov
            && piece == PieceKind::Pawn
            && (self[PieceKind::King] & self[!self.active])
                .contains(to.forward(self.active).unwrap())
            && !board.any_legal_move()
        {
            return false;
        }

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
    #[expect(clippy::inline_always, reason = "it improves performance")]
    #[inline(always)]
    pub fn gen_attackers(&self, sq: Square, side: Side) -> Bitboard {
        let occupancy = self.pieces.all();
        macro_rules! attackers {
            ($($bb:expr),* $(,)?) => {
                let mut attackers = Bitboard::EMPTY;
                $(attackers |= $bb;)*
                attackers & self[!side]
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

    pub fn legal_moves_with<R: Receiver>(&self, mask: Bitboard, r: R) -> R::Output {
        self.pseudolegal_moves_with(mask, receiver::Legal { board: self, recv: r })
    }

    pub fn legal_moves<R: Receiver>(&self, r: R) -> R::Output {
        self.pseudolegal_moves(receiver::Legal { board: self, recv: r })
    }

    pub fn pseudolegal_moves<R: Receiver>(&self, r: R) -> R::Output {
        self.pseudolegal_moves_with(Bitboard::FULL, r)
    }

    pub fn pseudolegal_moves_with<R: Receiver>(&self, mask: Bitboard, mut r: R) -> R::Output {
        let result = self.pseudolegal_moves_(&mut r, mask);
        r.finish(result)
    }
}
