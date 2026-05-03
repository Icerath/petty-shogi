//! format taking from <http://hgm.nubati.net/usi.html>

use std::io::Write as _;

use crate::{Board, File, Hand, Piece, PieceKind, Rank, Side, Square};

pub const INITIAL_SFEN: &str = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";

impl Board {
    #[expect(clippy::missing_panics_doc)]
    #[must_use]
    pub fn start_pos() -> Self {
        Self::from_sfen(INITIAL_SFEN).expect("the starting fen should be valid")
    }

    pub fn from_sfen(sfen: impl AsRef<[u8]>) -> Option<Self> {
        Self::from_split_sfen(sfen.as_ref().split(|b| *b == b' '))
    }

    pub fn from_split_sfen<'a>(mut fields: impl Iterator<Item = &'a [u8]>) -> Option<Self> {
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

    #[must_use]
    pub fn to_sfen(&self) -> String {
        let mut buf = vec![];
        self.write_fen(&mut buf);
        unsafe { String::from_utf8_unchecked(buf) }
    }

    pub fn write_fen(&self, buf: &mut Vec<u8>) {
        for rank in Rank::ALL {
            if rank as usize != 0 {
                buf.push(b'/');
            }
            let mut skipped = 0;
            for sq in rank.mask() {
                let Some(piece) = self.pieces.get(sq) else {
                    skipped += 1;
                    continue;
                };
                if skipped != 0 {
                    buf.push(skipped + b'0');
                }
                skipped = 0;
                _ = write!(buf, "{piece}");
            }
            if skipped != 0 {
                buf.push(skipped + b'0');
            }
        }

        buf.push(b' ');
        buf.push(if self.active == Side::Sente { b'b' } else { b'w' });

        buf.push(b' ');
        if self.hands.iter().flatten().sum::<u8>() == 0 {
            buf.push(b'-');
        } else {
            for side in Side::ALL {
                for &piece in PieceKind::ALL[..PieceKind::King as usize].iter().rev() {
                    match self.hands[side][piece] {
                        0 => continue,
                        1 => {}
                        count @ 2.. => buf.push(count + b'0'),
                    }
                    _ = write!(buf, "{}", Piece::new(side, piece, false));
                }
            }
        }

        _ = write!(buf, " {}", self.move_counter);
    }
}

fn parse_pieces(fen: &[u8]) -> Option<Board> {
    let mut board = Board::EMPTY;
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
    let mut fen = fen.iter().copied();
    while let Some(mut c) = fen.next() {
        let mut number = None;
        let kind = loop {
            break match c.to_ascii_lowercase() {
                b'0'..=b'9' => {
                    if number.is_some() {
                        return None;
                    }
                    number = Some(c - b'0');
                    // TODO: check for promoted flag
                    c = fen.next()?;
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
        hands[side][kind] = number.unwrap_or(1);
    }
    Some(hands)
}

#[test]
fn test_sfen() {
    assert_eq!(Board::EMPTY.to_sfen(), "9/9/9/9/9/9/9/9/9 b - 0");
    macro_rules! test {
        ($sfen: expr) => {
            assert_eq!(Board::from_sfen($sfen).unwrap().to_sfen(), $sfen);
        };
    }
    test!(INITIAL_SFEN);
    test!("+P3kgsnl/3sg2b1/4pp3/+R7p/3LP1p2/3K1PP2/P1PP4P/3b5/6+rNL w N5P2g2snlp 50");
}
