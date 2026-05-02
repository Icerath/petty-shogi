use std::{
    fmt::{self, Write as _},
    ops::{Index, IndexMut},
};

use crate::{Bitboard, File, Move, Piece, PieceKind, Rank, Side, Square, zobrist::Zobrist};

pub type Hand = [u8; PieceKind::LEN];

#[derive(Clone)]
pub struct Board {
    pub pieces: Pieces,
    pub hands: [Hand; 2],
    pub active: Side,
    pub move_counter: u32,
    pub zobrist: Zobrist,
}

impl Board {
    pub const EMPTY: Self = Self {
        pieces: Pieces::EMPTY,
        hands: [[0; PieceKind::LEN]; 2],
        active: Side::Sente,
        move_counter: 0,
        zobrist: Zobrist::EMPTY,
    };

    pub fn play(&mut self, mov: Move) {
        match mov {
            Move::Drop { piece, to } => self.drop_move(piece, to),
            Move::Board { from, to, promoted } => self.play_move(from, to, promoted),
        }
        self.zobrist.xor_side_to_move();
        self.active = !self.active;
        self.move_counter += 1;
    }

    fn drop_move(&mut self, piece: PieceKind, to: Square) {
        self.zobrist.xor_hand_piece(self.active, piece, self.hands[self.active][piece]);
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
            // debug_assert!(piece.kind() != PieceKind::King);
            self.zobrist.xor_hand_piece(
                self.active,
                piece.kind(),
                self.hands[self.active][piece.kind()],
            );
            self.hands[self.active][piece.kind()] += 1;
            self.remove_piece(piece, to);
        }

        self.remove_piece(from_piece, from);
        self.insert_piece(
            Piece::new(from_piece.side(), from_piece.kind(), from_piece.promoted() | promoted),
            to,
        );
    }

    fn remove_piece(&mut self, piece: Piece, sq: Square) {
        self[piece.side()].remove(sq);
        self[piece.kind()].remove(sq);
        self.pieces.promoted.remove(sq);
        self.zobrist.xor_board_piece(sq, piece);
    }

    pub fn insert_piece(&mut self, piece: Piece, sq: Square) {
        self[piece.side()].insert(sq);
        self[piece.kind()].insert(sq);
        if piece.promoted() {
            self.pieces.promoted.insert(sq);
        }
        self.zobrist.xor_board_piece(sq, piece);
    }
}

#[derive(Clone, PartialEq)]
#[expect(clippy::struct_field_names)]
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

    pub fn contains(&self, sq: Square) -> bool {
        self.all().contains(sq)
    }

    pub fn get(&self, sq: Square) -> Option<Piece> {
        let side = self.side(sq)?;
        let Some(kind) = self.kind(sq) else { unsafe { std::hint::unreachable_unchecked() } };
        let promoted = self.promoted.contains(sq);
        Some(Piece::new(side, kind, promoted))
    }

    pub fn kind(&self, sq: Square) -> Option<PieceKind> {
        PieceKind::ALL.into_iter().find(|&kind| self.pieces[kind].contains(sq))
    }

    pub fn side(&self, sq: Square) -> Option<Side> {
        if self.sides[Side::Sente].contains(sq) {
            Some(Side::Sente)
        } else if self.sides[Side::Gote].contains(sq) {
            Some(Side::Gote)
        } else {
            None
        }
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
            _ = writeln!(out, " {rank}");
        }
        out.push_str("+---+---+---+---+---+---+---+---+---+\n");
        write!(f, "{out}")
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Board {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Board {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str = <&str>::deserialize(deserializer)?;
        Self::from_sfen(str).ok_or_else(|| serde::de::Error::custom("invalid sfen"))
    }
}
