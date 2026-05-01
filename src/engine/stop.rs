use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

#[derive(Default, Clone)]
pub struct Stop {
    force: Arc<AtomicBool>,
    increment: u32,
    time_limit: Option<(Instant, Duration)>,
    infinite: bool,
}

impl Stop {
    pub fn reset(&mut self) {
        self.force.store(false, Ordering::Relaxed);
        *self = Self { force: self.force.clone(), ..Self::default() };
    }

    pub fn infinite(&mut self, infinite: bool) -> &mut Self {
        self.infinite = infinite;
        self
    }

    pub fn time_limit(&mut self, start: Instant, duration: Duration) -> &mut Self {
        self.time_limit = Some((start, duration));
        self
    }

    pub fn set_stop(&self) {
        self.force.store(true, Ordering::Relaxed);
    }

    pub fn is_stop(&mut self) -> bool {
        if self.increment < 1024 {
            self.increment += 1;
            return false;
        }
        self.get_is_stop()
    }

    #[cold]
    #[inline(never)]
    fn get_is_stop(&mut self) -> bool {
        if self.infinite {
            self.increment = 0;
            return false;
        }
        if self.force.load(Ordering::Relaxed)
            || self.time_limit.as_ref().is_some_and(|(start, duration)| start.elapsed() > *duration)
        {
            return true;
        }
        self.increment = 0;
        false
    }
}
