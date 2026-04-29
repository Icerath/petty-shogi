pub mod command;
pub mod response;
mod score;
mod search;
mod stop;

use std::{
    ops::ControlFlow,
    sync::Arc,
    time::{Duration, Instant},
};

use command::{Command, GoCommand, Position};
use response::{BestMove, Response};
use score::Score;
use search::Search;
use stop::Stop;

use crate::{Action, Board, Side};

pub struct Engine<R> {
    board: Board,
    search: Search,
    recv: Arc<R>,
    stop: Stop,
}

impl<R> Engine<R>
where
    R: Fn(Response),
{
    pub fn init(recv: R) -> Self {
        Self {
            board: Board::start_pos(),
            search: Search::default(),
            recv: Arc::new(recv),
            stop: Stop::default(),
        }
    }

    pub fn process_command(&mut self, command: Command)
    where
        R: Send + Sync + 'static,
    {
        match command {
            Command::Usi => {
                self.recv(Response::Id(response::Id::Name("PettyShogi".into())));
                self.recv(Response::Id(response::Id::Author("Dorje Gilfillan".into())));
                self.recv(Response::UsiOk);
            }
            Command::IsReady => self.recv(Response::ReadyOk),
            Command::Go(go) => self.go(go),
            Command::UsiNewGame => {}
            Command::Position(position, moves) => self.position(position, moves),
            Command::Stop => self.stop(),
            Command::Display => self.recv(Response::Misc(self.board.to_string())),
            command => todo!("{command:?}"),
        }
    }

    pub fn stop(&self) {
        self.stop.set_stop();
    }

    pub fn position(&mut self, position: Position, moves: Vec<Action>) {
        match position {
            Position::Sfen(sfen) => match Board::from_sfen(sfen) {
                Some(board) => self.board = board,
                None => self.recv(Response::Error("Invalid SFEN".into())),
            },
            Position::StartPos => self.board = Board::start_pos(),
        }
        for mov in moves {
            if self.board.has_legal_move(mov) {
                self.board.play(mov);
            } else {
                self.recv(Response::Error(format!("cannot play {mov}")));
                break;
            }
        }
    }

    fn go(&self, go: GoCommand)
    where
        R: Send + Sync + 'static,
    {
        if Arc::strong_count(&self.recv) > 1 {
            self.recv(Response::Error(
                "you must stop the previous go command before calling go again".into(),
            ));
            return;
        }
        let mut engine = Engine {
            board: self.board.clone(),
            search: self.search.clone(),
            recv: self.recv.clone(),
            stop: self.stop.clone(),
        };
        std::thread::spawn(move || engine.go_blocking(go));
    }

    fn go_blocking(&mut self, go: GoCommand) {
        self.stop.reset();
        if let Some(time) = go.movetime {
            self.stop.time_limit(Instant::now(), Duration::from_millis(time as u64));
        }
        self.stop.infinite(go.infinite);

        let max_depth = go.depth.unwrap_or(u32::MAX);

        if let Some(perft) = go.perft {
            self.perft(perft);
            return;
        }
        self.search = Search::default();

        let start = Instant::now();
        let mut complete_line = vec![];
        for depth in 1..max_depth {
            let mut line = vec![];
            let ControlFlow::Continue(score) = self.search(&self.board.clone(), depth, &mut line)
            else {
                break;
            };
            line.reverse();
            self.recv(Response::Info {
                depth,
                time: start.elapsed().as_millis() as u32,
                nodes: self.search.nodes,
                score,
                line: line.clone(),
            });
            complete_line = line;
        }
        if let Some(&best_move) = complete_line.first() {
            self.recv(Response::BestMove(BestMove::Move { mov: best_move, ponder: None }))
        } else {
            self.recv(Response::BestMove(BestMove::Resign))
        }
    }

    fn perft(&mut self, depth: u32) {
        if depth == 0 {
            self.recv(Response::Misc("Found 0 positions in 0s".to_string()));
            return;
        }
        let start = Instant::now();
        let mut sum = 0;
        let mut stop = self.stop.clone();
        let result = self.board.legal_moves(|mov| {
            let mut board = self.board.clone();
            board.play(mov);
            let positions = board.try_perft(depth - 1, &mut || stop.is_stop())?;
            self.recv(Response::Misc(format!("{mov}: {positions}")));
            sum += positions;
            ControlFlow::Continue(())
        });
        if result.is_continue() {
            self.recv(Response::Misc(format!("Found {sum} positions in {:.0?}", start.elapsed())));
        }
    }

    fn recv(&self, response: Response) {
        (self.recv)(response)
    }
}
