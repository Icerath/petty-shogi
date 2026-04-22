use std::{
    fmt::{self, Write as _},
    ops::{Index, IndexMut},
};

use crate::{Action, Bitboard, File, Piece, PieceKind, Rank, Side, Square};

pub type Hand = [u8; PieceKind::LEN];

#[derive(Clone)]
pub struct Board {
    pub pieces: Pieces,
    pub hands: [Hand; 2],
    pub active: Side,
    pub move_counter: u32,
}

impl Board {
    pub const EMPTY: Self = Self {
        pieces: Pieces::EMPTY,
        hands: [[0; PieceKind::LEN]; 2],
        active: Side::Sente,
        move_counter: 0,
    };

    pub fn play(&mut self, action: Action) {
        match action {
            Action::Drop { piece, to } => self.drop_move(piece, to),
            Action::Move { from, to, promoted } => self.play_move(from, to, promoted),
        }
        self.active = !self.active;
    }

    fn drop_move(&mut self, piece: PieceKind, to: Square) {
        self.hands[self.active][piece] -= 1;
        self.insert_piece(Piece::new(self.active, piece, false), to);
    }

    fn play_move(&mut self, from: Square, to: Square, promoted: bool) {
        let from_piece = Piece::new(
            self.active,
            self.pieces.kind(from).unwrap(),
            self.pieces.promoted.contains(from),
        );
        debug_assert_eq!(from_piece.side(), self.active);

        if let Some(piece) = self.pieces.get(to) {
            debug_assert!(piece.kind() != PieceKind::King);
            self.hands[self.active][piece.kind()] += 1;
            self.remove_piece(piece, to);
        }

        self.remove_piece(from_piece, from);
        self.insert_piece(from_piece, to);
        if promoted {
            debug_assert!(!from_piece.promoted());
            debug_assert!(from_piece.kind() != PieceKind::Gold);
            debug_assert!(from.is_promotion_zone(self.active) || to.is_promotion_zone(self.active));
            self.pieces.promoted.insert(to);
        }
    }

    fn remove_piece(&mut self, piece: Piece, sq: Square) {
        self[piece.side()].remove(sq);
        self[piece.kind()].remove(sq);
        self.pieces.promoted.remove(sq);
    }

    pub fn insert_piece(&mut self, piece: Piece, sq: Square) {
        self[piece.side()].insert(sq);
        self[piece.kind()].insert(sq);
        if piece.promoted() {
            self.pieces.promoted.insert(sq);
        }
    }
}

#[derive(Clone)]
pub struct Pieces {
    pub sides: [Bitboard; 2],
    pub promoted: Bitboard,
    pub pieces: [Bitboard; PieceKind::LEN],
}

impl Pieces {
    pub const EMPTY: Self = Self {
        sides: [Bitboard::EMPTY; 2],
        promoted: Bitboard::EMPTY,
        pieces: [Bitboard::EMPTY; PieceKind::LEN],
    };

    pub fn all(&self) -> Bitboard {
        self.sides[0] | self.sides[1]
    }

    pub fn get(&self, sq: Square) -> Option<Piece> {
        let side = if self.sides[0].contains(sq) {
            Side::Sente
        } else if self.sides[1].contains(sq) {
            Side::Gote
        } else {
            return None;
        };
        let Some(kind) = self.kind(sq) else { unreachable!() };
        let promoted = self.promoted.contains(sq);
        Some(Piece::new(side, kind, promoted))
    }

    pub fn kind(&self, sq: Square) -> Option<PieceKind> {
        PieceKind::ALL.into_iter().find(|&kind| self.pieces[kind].contains(sq))
    }
}

impl Index<PieceKind> for Board {
    type Output = Bitboard;

    fn index(&self, index: PieceKind) -> &Self::Output {
        &self.pieces.pieces[index]
    }
}

impl IndexMut<PieceKind> for Board {
    fn index_mut(&mut self, index: PieceKind) -> &mut Self::Output {
        &mut self.pieces.pieces[index]
    }
}

impl Index<Side> for Board {
    type Output = Bitboard;

    fn index(&self, index: Side) -> &Self::Output {
        &self.pieces.sides[index]
    }
}

impl IndexMut<Side> for Board {
    fn index_mut(&mut self, index: Side) -> &mut Self::Output {
        &mut self.pieces.sides[index]
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        out.push_str("  9   8   7   6   5   4   3   2   1\n");
        for rank in 0..9 {
            let rank = Rank::from_int(rank).unwrap();
            out.push_str("+---+---+---+---+---+---+---+---+---+\n|");
            for file in 0..9 {
                let square = Square::new(File::from_int(file).unwrap(), rank);
                match self.pieces.get(square) {
                    Some(piece) if !piece.promoted() => write!(out, " {piece}")?,
                    Some(piece) => write!(out, "{piece}")?,
                    None => out.push_str("  "),
                }
                out.push_str(" |");
            }
            _ = writeln!(out, " {}", rank);
        }
        out.push_str("+---+---+---+---+---+---+---+---+---+\n");
        write!(f, "{out}")
    }
}
