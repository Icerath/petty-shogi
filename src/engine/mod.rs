pub mod command;
mod move_ordering;
mod movelist;
mod piece_value;
pub mod response;
mod score;
mod search;
mod stop;
mod transposition_table;

use std::{
    ops::ControlFlow,
    sync::{Arc, Condvar, Mutex},
    time::{Duration, Instant},
};

use command::{Command, GoCommand, Position};
use response::{BestMove, Info, Response};
use score::Score;
use search::Search;
use stop::Stop;
use transposition_table::TTable;

use crate::{Board, Move};

pub struct Engine {
    position: Board,
    search: Search,
    recv: Option<Arc<dyn Fn(Response) + 'static + Send + Sync>>,
    wait: Arc<(Mutex<bool>, Condvar)>,
    stop: Stop,
    ttable: transposition_table::TTable,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            position: Board::start_pos(),
            search: Search::default(),
            recv: None,
            wait: Arc::new((Mutex::new(false), Condvar::new())),
            stop: Stop::default(),
            ttable: TTable::from_bytes(64 * 1024 * 1024),
        }
    }
}

impl Engine {
    pub fn set_recv(&mut self, recv: impl Fn(Response) + 'static + Send + Sync) {
        self.recv = Some(Arc::new(recv));
    }

    pub fn process_command(&mut self, command: Command) {
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
            Command::Display => self.recv(Response::Misc(self.position.to_string())),
            command => todo!("{command:?}"),
        }
    }

    pub fn stop(&self) {
        self.stop.set_stop();
    }

    pub fn position(&mut self, position: Position, moves: Vec<Move>) {
        match position {
            Position::Sfen(sfen) => match Board::from_sfen(sfen) {
                Some(board) => self.position = board,
                None => self.recv(Response::Error("Invalid SFEN".into())),
            },
            Position::StartPos => self.position = Board::start_pos(),
        }
        for mov in moves {
            if self.position.is_legal(mov) {
                self.position.play(mov);
            } else {
                self.recv(Response::Error(format!("cannot play {mov}")));
                break;
            }
        }
    }

    #[expect(clippy::missing_panics_doc)]
    /// blocks until the current go command finishes
    pub fn wait(&self) {
        loop {
            let guard = self.wait.0.lock().unwrap();
            if !*self.wait.1.wait(guard).unwrap() {
                break;
            }
        }
    }

    fn go(&self, go: GoCommand) {
        if *self.wait.0.lock().unwrap() {
            self.recv(Response::Error(
                "you must stop the previous go command before calling go again".into(),
            ));
            return;
        }
        let mut engine = Self {
            position: self.position.clone(),
            search: self.search.clone(),
            recv: self.recv.clone(),
            stop: self.stop.clone(),
            ttable: self.ttable.clone(),
            wait: self.wait.clone(),
        };
        std::thread::spawn(move || engine.go_blocking(&go));
    }

    fn go_blocking(&mut self, go: &GoCommand) {
        *self.wait.0.lock().unwrap() = true;
        self.stop.reset();
        if let Some(time) = go.movetime {
            self.stop.time_limit(Instant::now(), Duration::from_millis(time.into()));
        }
        self.stop.infinite(go.infinite);

        let max_depth = go.depth.unwrap_or(u32::MAX);

        if let Some(perft) = go.perft {
            self.perft(perft);
            *self.wait.0.lock().unwrap() = false;
            self.wait.1.notify_all();
            return;
        }
        self.search = Search::default();

        let start = Instant::now();
        let mut complete_line = vec![];
        for depth in 1..=max_depth {
            self.search.max_seldepth = 0;
            let mut line = vec![];
            let score = self.search_root(&mut self.position.clone(), depth, &mut line);
            if self.stop.is_stop() {
                break;
            }
            self.recv(Response::Info(
                Info::default()
                    .depth(depth)
                    .seldepth(self.search.max_seldepth)
                    .time(u32::try_from(start.elapsed().as_millis()).unwrap_or(u32::MAX))
                    .nodes(self.search.nodes)
                    .hashfull(self.ttable.hashfull())
                    .score(score)
                    .line(line.clone()),
            ));
            self.recv(Response::Verbose(format!(
                "fail high: {:.1}%",
                (self.search.fail_high * 1000 / self.search.fail_high_test) as f64 / 10.0
            )));
            complete_line = line;
            if score.mate().is_some() {
                break;
            }
        }
        if let Some(&best_move) = complete_line.first() {
            self.recv(Response::BestMove(BestMove::Move { mov: best_move, ponder: None }));
        } else {
            self.recv(Response::BestMove(BestMove::Resign));
        }
        *self.wait.0.lock().unwrap() = false;
        self.wait.1.notify_all();
    }

    fn perft(&self, depth: u32) {
        if depth == 0 {
            self.recv(Response::Misc("Found 0 positions in 0s".to_string()));
            return;
        }
        let start = Instant::now();
        let mut sum = 0;
        let mut stop = self.stop.clone();
        let result = self.position.legal_moves(|mov| {
            let mut board = self.position.clone();
            board.play(mov);
            let positions = board.try_perft(depth - 1, &mut |_| stop.is_stop())?;
            self.recv(Response::Misc(format!("{mov}: {positions}")));
            sum += positions;
            ControlFlow::Continue(())
        });
        if result.is_continue() {
            self.recv(Response::Misc(format!("Found {sum} positions in {:.0?}", start.elapsed())));
        }
    }

    fn recv(&self, response: Response) {
        if let Some(recv) = &self.recv {
            recv(response);
        }
    }
}
