use std::{cell::UnsafeCell, sync::Arc};

use crate::zobrist::Zobrist;

pub struct TTable<T> {
    entries: Arc<[UnsafeCell<Option<T>>]>,
}

impl<T> Clone for TTable<T> {
    fn clone(&self) -> Self {
        Self { entries: self.entries.clone() }
    }
}

impl<T> TTable<T> {
    pub fn from_bytes(bytes: usize) -> Self {
        Self::from_num_entries(bytes / size_of::<T>())
    }

    pub fn from_num_entries(capacity: usize) -> Self {
        Self { entries: std::iter::repeat_with(|| UnsafeCell::new(None)).take(capacity).collect() }
    }

    #[expect(clippy::mut_from_ref, reason = "lol to ub")]
    #[expect(clippy::cast_possible_truncation)]
    pub fn get_option(&self, zobrist: Zobrist) -> &mut Option<T> {
        unsafe {
            self.entries[(zobrist.0 % self.entries.len() as u64) as usize].get().as_mut_unchecked()
        }
    }

    pub fn get(&self, zobrist: Zobrist) -> Option<&mut T> {
        self.get_option(zobrist).as_mut()
    }

    pub fn capacity(&self) -> usize {
        self.entries.len()
    }
}

unsafe impl<T> Send for TTable<T> {}
unsafe impl<T> Sync for TTable<T> {}
