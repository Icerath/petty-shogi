//! format taking from http://hgm.nubati.net/usi.html

use crate::{
    board::{Board, Hand},
    piece::{Piece, PieceKind},
    side::Side,
    square::{File, Rank, Square},
};

pub const INITIAL_SFEN: &str = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";

impl Board {
    pub fn start_pos() -> Self {
        Self::from_sfen(INITIAL_SFEN.as_ref()).expect("the starting fen should be valid")
    }

    pub fn from_sfen(sfen: &[u8]) -> Option<Self> {
        let mut fields = sfen.as_ref().split(|&b| b == b' ');
        let mut board = parse_pieces(fields.next()?)?;
        board.active = match fields.next()? {
            b"b" => Side::Sente,
            b"w" => Side::Gote,
            _ => return None,
        };
        board.hands = parse_hands(fields.next()?)?;
        board.move_counter = fields.next().and_then(atoi::atoi).unwrap_or(0);
        Some(board)
    }

    pub fn into_sfen(&self) -> Vec<u8> {
        todo!()
    }
}

fn parse_pieces(fen: &[u8]) -> Option<Board> {
    let mut board = Board::default();
    let mut rank = 0;
    let mut file = 0;

    let mut promoted = false;
    for c in fen {
        let kind = match c.to_ascii_lowercase() {
            b'1'..=b'9' => {
                // TODO: check for promoted flag
                file += c - b'0';
                continue;
            }
            b'/' => {
                // TODO: check for promoted flag
                file = 0;
                rank += 1;
                continue;
            }
            b'+' => {
                // TODO: check for promoted flag
                promoted = true;
                continue;
            }
            b'p' => PieceKind::Pawn,
            b'l' => PieceKind::Lance,
            b'n' => PieceKind::Knight,
            b's' => PieceKind::Silver,
            b'g' => PieceKind::Gold,
            b'b' => PieceKind::Bishop,
            b'r' => PieceKind::Rook,
            b'k' => PieceKind::King,
            _ => return None,
        };
        let side = if c.is_ascii_uppercase() { Side::Sente } else { Side::Gote };
        let sq = Square::new(File::from_int(file)?, Rank::from_int(rank)?);
        board.insert_piece(Piece::new(side, kind, promoted), sq);
        file += 1;
        promoted = false;
    }
    Some(board)
}

fn parse_hands(fen: &[u8]) -> Option<[Hand; 2]> {
    let mut hands = [[0; PieceKind::LEN]; 2];
    if fen == b"-" {
        return Some(hands);
    }
    for c in fen {
        let mut number = None;
        let kind = loop {
            break match c.to_ascii_lowercase() {
                b'0'..=b'9' => {
                    number = Some(c - b'0');
                    // TODO: check for promoted flag
                    continue;
                }
                b'p' => PieceKind::Pawn,
                b'l' => PieceKind::Lance,
                b'n' => PieceKind::Knight,
                b's' => PieceKind::Silver,
                b'g' => PieceKind::Gold,
                b'b' => PieceKind::Bishop,
                b'r' => PieceKind::Rook,
                _ => return None,
            };
        };
        let side = if c.is_ascii_uppercase() { Side::Sente } else { Side::Gote };
        hands[side as usize][kind as usize] = number.unwrap_or(1);
    }
    Some(hands)
}
