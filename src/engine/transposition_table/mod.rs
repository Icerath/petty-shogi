use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use super::score::Score;
use crate::zobrist::Zobrist;

mod raw;

#[derive(Clone)]
pub struct TTable {
    raw: raw::TTable<TEntry>,
    len: Arc<AtomicUsize>,
    pub num_hits: u64,
}

impl TTable {
    pub fn from_bytes(bytes: usize) -> Self {
        Self {
            raw: raw::TTable::from_bytes(bytes),
            num_hits: 0,
            len: Arc::new(AtomicUsize::new(0)),
        }
    }

    #[must_use]
    pub fn get(
        &mut self,
        zobrist: Zobrist,
        alpha: Score,
        beta: Score,
        depth: u32,
    ) -> Option<&TEntry> {
        let entry = self.raw.get(zobrist)?;
        if entry.zobrist != zobrist || entry.depth < depth {
            return None;
        }
        if (entry.nodetype == Nodetype::Exact)
            || (entry.nodetype == Nodetype::Alpha && entry.score <= alpha)
            || (entry.nodetype == Nodetype::Beta && entry.score >= beta)
        {
            self.num_hits += 1;
            return Some(entry);
        }
        None
    }

    pub fn insert(&mut self, zobrist: Zobrist, depth: u32, score: Score, nodetype: Nodetype) {
        if score.mate().is_some() {
            return;
        }

        let new_entry = TEntry { zobrist, depth, score, nodetype };
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
    pub nodetype: Nodetype,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Nodetype {
    Exact,
    Alpha,
    Beta,
}
