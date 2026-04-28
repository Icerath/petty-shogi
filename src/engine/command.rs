use std::str::FromStr;

use crate::Action;

#[derive(Debug)]
pub enum Command {
    Usi,
    UsiNewGame,
    IsReady,
    Position(Position, Vec<Action>),
    Go(GoCommand),
    Stop,
    PonderHit,
    GameOver(GameOver),
    Quit,
}

#[derive(Debug, Default)]
pub struct GoCommand {
    pub search_moves: Vec<Action>,
    pub ponder: bool,
    pub btime: Option<u64>,
    pub wtime: Option<u64>,
    pub byoyomi: Option<u64>,
    pub movestogo: Option<u32>,
    pub depth: Option<u32>,
    pub nodes: Option<u32>,
    pub mate: Option<u32>,
    pub movetime: Option<u32>,
    pub infinite: bool,
    pub perft: Option<u32>,
}

#[derive(Debug)]
pub enum Position {
    Sfen(String),
    StartPos,
}
#[derive(Debug, Clone, Copy)]
pub enum GameOver {
    Win,
    Lose,
    Draw,
}

impl FromStr for Command {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split(' ');
        Ok(match split.next().ok_or(())? {
            "usi" => Self::Usi,
            "quit" => Self::Quit,
            "stop" => Self::Stop,
            "go" => Self::Go(GoCommand::from_split(split)?),
            _ => return Err(()),
        })
    }
}

impl FromStr for GoCommand {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_split(s.split(' '))
    }
}

impl GoCommand {
    fn from_split<'a>(mut split: impl Iterator<Item = &'a str>) -> Result<Self, ()> {
        macro_rules! parse_int {
            () => {{
                let Some(value) = split.next() else { continue };
                let Ok(value) = value.parse::<u32>() else { continue };
                value
            }};
        }

        let mut builder = Self::default();
        while let Some(next) = split.next() {
            match next {
                "depth" => builder.depth = Some(parse_int!()),
                "perft" => builder.perft = Some(parse_int!()),
                "movetime" => builder.movetime = Some(parse_int!()),
                _ => {}
            }
        }
        Ok(builder)
    }
}
