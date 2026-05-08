use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use super::score::Score;
use crate::{Move, zobrist::Zobrist};

mod raw;

#[derive(Clone)]
pub struct TTable {
    raw: raw::TTable<TEntry>,
    len: Arc<AtomicUsize>,
}

impl TTable {
    pub fn from_bytes(bytes: usize) -> Self {
        Self { raw: raw::TTable::from_bytes(bytes), len: Arc::new(AtomicUsize::new(0)) }
    }

    #[must_use]
    pub fn get(&mut self, zobrist: Zobrist) -> Option<&TEntry> {
        let entry = self.raw.get(zobrist)?;
        if entry.zobrist != zobrist {
            return None;
        }
        Some(entry)
    }

    pub fn insert(
        &mut self,
        zobrist: Zobrist,
        depth: u32,
        score: Score,
        mov: Option<Move>,
        nodetype: Nodetype,
    ) {
        if score.mate().is_some() {
            return;
        }
        let new_entry = TEntry { zobrist, depth, score, mov, nodetype };
        match self.raw.get_option(zobrist) {
            Some(occupied) if occupied.depth <= depth => *occupied = new_entry,
            Some(_) => {}
            opt @ None => {
                *opt = Some(new_entry);
                self.len.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    #[expect(clippy::cast_possible_truncation)]
    pub fn hashfull(&self) -> u32 {
        let len = self.len.load(Ordering::Relaxed);
        ((len * 1_000_000) / self.raw.capacity()) as u32
    }
}

pub struct TEntry {
    pub zobrist: Zobrist,
    pub depth: u32,
    pub score: Score,
    pub mov: Option<Move>,
    pub nodetype: Nodetype,
}

impl TEntry {
    pub fn score(&self, alpha: Score, beta: Score, depth: u32) -> Option<Score> {
        if self.depth < depth {
            return None;
        }
        if (self.nodetype == Nodetype::Exact)
            || (self.nodetype == Nodetype::Alpha && self.score <= alpha)
            || (self.nodetype == Nodetype::Beta && self.score >= beta)
        {
            return Some(self.score);
        }
        None
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Nodetype {
    Exact,
    Alpha,
    Beta,
}
