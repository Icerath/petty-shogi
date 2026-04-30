use crate::{Action, Board};

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
    /// not part of the USI spec
    Display,
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
    /// not part of the USI spec
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

impl Command {
    pub fn from_usi(str: &str) -> Option<Self> {
        let mut words = str.split(' ');
        Some(match words.next()? {
            "usi" => Self::Usi,
            "isready" => Self::IsReady,
            "usinewgame" => Self::UsiNewGame,
            "position" => {
                let (position, moves) = parse_position(words)?;
                Self::Position(position, moves)
            }
            "quit" => Self::Quit,
            "stop" => Self::Stop,
            "go" => Self::Go(GoCommand::from_split(words)),
            "display" => Self::Display,
            _ => return None,
        })
    }
}

impl GoCommand {
    fn from_split<'a>(mut split: impl Iterator<Item = &'a str>) -> Self {
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
        builder
    }
}

fn parse_position<'a>(mut words: impl Iterator<Item = &'a str>) -> Option<(Position, Vec<Action>)> {
    let position = match words.next()? {
        "startpos" => Position::StartPos,
        "sfen" => {
            Position::Sfen(Board::from_split_sfen(words.by_ref().map(str::as_bytes))?.to_sfen())
        }
        _ => return None,
    };
    let mut moves = vec![];
    if let Some("moves") = words.next()
        && let Ok(parsed) = words.map(|word| word.parse()).collect::<Result<Vec<_>, _>>()
    {
        moves = parsed;
    }
    Some((position, moves))
}
