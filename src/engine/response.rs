use std::fmt;

use super::score::Score;
use crate::Action;

pub enum Response {
    Id(Id),
    UsiOk,
    ReadyOk,
    BestMove(BestMove),
    Info { depth: u32, time: u32, nodes: u64, score: Score, line: Vec<Action> },
    Error(String),
    Misc(String),
}

pub enum Id {
    Name(String),
    Author(String),
}

#[derive(Clone, Copy)]
pub enum BestMove {
    Resign,
    Win,
    Move { mov: Action, ponder: Option<Action> },
}

impl fmt::Display for Response {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Id(ref id) => write!(f, "id {id}"),
            Self::UsiOk => write!(f, "usiok"),
            Self::ReadyOk => write!(f, "readyok"),
            Self::BestMove(best_move) => write!(f, "bestmove {best_move}"),
            Self::Info { depth, time, nodes, score, ref line } => {
                write!(
                    f,
                    "info depth {depth} time {time} nodes {nodes}{nps} cp {score} pv ",
                    nps = display_nps(nodes, time),
                )?;
                for mov in line {
                    write!(f, "{mov} ")?;
                }
                Ok(())
            }
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

fn display_nps(nodes: u64, millis: u32) -> impl fmt::Display {
    fmt::from_fn(move |f| {
        if millis == 0 { Ok(()) } else { write!(f, " nps {}", (nodes * 1000) / millis as u64) }
    })
}
