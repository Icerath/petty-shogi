use crate::{File, Piece, PieceKind, Rank, Side, Square};

macro_rules! impl_index {
    ($($ty:ty),*) => {
        $(impl<T> core::ops::Index<$ty> for [T; <$ty>::LEN] {
            type Output = T;
            fn index(&self, index: $ty) -> &T {
                unsafe { self.get_unchecked(index as usize) }
            }
        }
        impl<T> core::ops::IndexMut<$ty> for [T; <$ty>::LEN] {
            fn index_mut(&mut self, index: $ty) -> &mut T {
                unsafe { self.get_unchecked_mut(index as usize) }
            }
        })*
    };
}

impl_index!(Side, PieceKind, Piece, Square, Rank, File);
