pub mod magic;
mod pseudolegal;
mod receiver;

use std::ops::ControlFlow;

use magic::{bishop_moves, lance_moves, rook_moves};
use pseudolegal::{GOLD_LUT, KING_LUT, KNIGHT_LUT, SILVER_LUT};
pub use receiver::Receiver;

use crate::{Bitboard, Board, Move, PieceKind, Side, Square, board_state::BoardState};

impl<S: BoardState> Board<S> {
    fn any_legal_move(&self) -> bool {
        self.legal_moves(|_| ControlFlow::Break(())).is_break()
    }

    #[must_use]
    pub fn is_legal(&self, mov: Move) -> bool {
        self.without_state().is_pseudolegal(mov) && self.is_generated_legal(mov)
    }

    #[must_use]
    pub(crate) fn is_generated_legal(&self, mov: Move) -> bool {
        let mut board = self.without_state().clone();
        board.play(mov);
        board.was_legal(mov)
    }

    // checks if a move was legal after playing
    #[must_use]
    pub(crate) fn was_legal(&self, mov: Move) -> bool {
        // check for pawn drop mate
        if let Move::Drop { piece, to } = mov
            && piece == PieceKind::Pawn
            && (self[PieceKind::King] & self[self.active])
                .contains(unsafe { to.back_unchecked(self.active) })
            && !self.any_legal_move()
        {
            return false;
        }

        let Some(king_square) = (self[PieceKind::King] & self[!self.active]).bitscan() else {
            return true;
        };
        self.gen_attackers(king_square, !self.active).is_empty()
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
            GOLD_LUT[side][sq] & self.without_state().gold_move_pieces(),
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
        self.pseudolegal_moves_with(mask, receiver::Legal { board: self.without_state(), recv: r })
    }

    pub fn legal_moves<R: Receiver>(&self, r: R) -> R::Output {
        self.pseudolegal_moves(receiver::Legal { board: self.without_state(), recv: r })
    }

    pub fn pseudolegal_moves<R: Receiver>(&self, r: R) -> R::Output {
        self.pseudolegal_moves_with(Bitboard::FULL, r)
    }

    pub fn pseudolegal_moves_with<R: Receiver>(&self, mask: Bitboard, r: R) -> R::Output {
        self.pseudolegal_moves_all(mask & !self[self.active], r)
    }

    pub fn pseudolegal_moves_all<R: Receiver>(&self, mask: Bitboard, mut r: R) -> R::Output {
        let result = self.without_state().pseudolegal_moves_(&mut r, mask);
        r.finish(result)
    }
}
