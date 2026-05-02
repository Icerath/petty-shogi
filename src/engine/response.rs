use std::fmt;

use super::score::Score;
use crate::Move;

#[derive(Debug)]
pub enum Response {
    Id(Id),
    UsiOk,
    ReadyOk,
    BestMove(BestMove),
    Info(Info),
    // not part of USI, should be printed to stderr instead of stdout
    Error(String),
    // not part of USI, should be printed to stderr instead of stdout
    Misc(String),
}

#[derive(Debug)]
pub enum Id {
    Name(String),
    Author(String),
}

macro_rules! define_info {
    ($($field:ident: $ty:ty,)* $(,)?) => {
        #[derive(Debug, Default)]
        pub struct Info {
            $($field: Option<$ty>),*
        }
        impl Info {
            $(#[must_use] pub fn $field(mut self, $field: $ty) -> Self {
                self.$field = Some($field);
                self
            })*
        }
    };
}

define_info! {
    depth: u32,
    time: u32,
    nodes: u64,
    score: Score,
    line: Vec<Move>,
    hashfull: u32,
}

#[derive(Debug, Clone, Copy)]
pub enum BestMove {
    Resign,
    Win,
    Move { mov: Move, ponder: Option<Move> },
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Id(ref id) => write!(f, "id {id}"),
            Self::UsiOk => write!(f, "usiok"),
            Self::ReadyOk => write!(f, "readyok"),
            Self::BestMove(best_move) => write!(f, "bestmove {best_move}"),
            Self::Info(ref info) => write!(f, "info {info}"),
            Self::Misc(ref string) => write!(f, "{string}"),
            Self::Error(..) => Ok(()), // can I display errors?
        }
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Name(name) => write!(f, "name {name}"),
            Self::Author(author) => write!(f, "author {author}"),
        }
    }
}

impl fmt::Display for BestMove {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Win => write!(f, "win"),
            Self::Resign => write!(f, "resign"),
            Self::Move { mov, ponder } => {
                write!(f, "{mov}")?;
                if let Some(ponder) = ponder {
                    write!(f, "ponder {ponder}")?;
                }
                Ok(())
            }
        }
    }
}

impl fmt::Display for Info {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { depth, time, nodes, score, ref line, hashfull } = *self;
        let nps = time
            .filter(|time| *time != 0)
            .and_then(|time| nodes.map(|nodes| nodes * 1000 / u64::from(time)));

        macro_rules! display {
            ($($ident:ident),* $(,)?) => {
                $(if let Some(t) = $ident {
                    write!(f, "{} {t} ", stringify!($ident))?;
                })*
            };
        }

        display!(depth, score, time, nodes, nps, hashfull);

        if let Some(line) = line {
            write!(f, "pv ")?;
            for mov in line {
                write!(f, "{mov} ")?;
            }
        }
        Ok(())
    }
}
