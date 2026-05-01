use std::{cmp::Ordering, fmt, ops::Neg};

#[derive(Debug, Clone, Copy)]
pub struct Score(pub i32);

impl Score {
    pub const MATE: Self = Self(i32::MAX - 1);
    pub const MAX: Self = Self(i32::MAX);

    pub fn step(mut self) -> Self {
        if self.mate().is_some() {
            self.0 -= self.0.signum();
        }
        self
    }

    pub fn mate(self) -> Option<i32> {
        if !((Self::MATE.0 - 1000)..=Self::MATE.0).contains(&self.0.abs()) {
            return None;
        }
        let sign = self.0.signum();
        Some((Self::MATE.0 - self.0.abs()) * sign)
    }

    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }
}

impl Neg for Score {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl PartialEq for Score {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
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
        self.0.cmp(&other.0)
    }
}

impl fmt::Display for Score {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.mate() {
            Some(ply) => write!(f, "mate {ply}"),
            None => write!(f, "cp {}", self.0),
        }
    }
}

#[test]
fn test_mate_values() {
    assert_eq!(Score::MATE.mate(), Some(0));
    assert_eq!(Score::MATE.step().mate(), Some(1));
    assert_eq!((-Score::MATE).step().mate(), Some(-1));
}
