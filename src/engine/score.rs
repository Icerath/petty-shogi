use std::{cmp::Ordering, fmt};

#[derive(Clone, Copy)]
pub enum Score {
    CentiPawns(i32),
    Mate(i32),
}

impl Score {
    pub fn step(self) -> Self {
        match self {
            Self::CentiPawns(score) => Self::CentiPawns(-score),
            Self::Mate(ply @ 1..) => Self::Mate(ply + 1),
            Self::Mate(ply @ ..=0) => Self::Mate(ply - 1),
        }
    }
}

impl PartialEq for Score {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other).is_eq()
    }
}

impl Eq for Score {}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Mate(l), Self::Mate(r)) => r.cmp(l),
            (Self::CentiPawns(l), Self::CentiPawns(r)) => l.cmp(r),
            (Self::Mate(1..), Self::CentiPawns(_)) => Ordering::Greater,
            (Self::Mate(..=0), Self::CentiPawns(_)) => Ordering::Less,
            (Self::CentiPawns(_), Self::Mate(..=0)) => Ordering::Greater,
            (Self::CentiPawns(_), Self::Mate(_)) => Ordering::Less,
        }
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CentiPawns(cp) => write!(f, "{cp}"),
            Self::Mate(plies) => write!(f, "mate {plies}"),
        }
    }
}
