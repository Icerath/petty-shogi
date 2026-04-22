use crate::{File, Piece, PieceKind, Rank, Side, Square};

macro_rules! impl_index {
    ($($ty:ty),*) => {
        $(impl<T> core::ops::Index<$ty> for [T; <$ty>::LEN] {
            type Output = T;
            #[inline(always)]
            fn index(&self, index: $ty) -> &T {
                unsafe { self.get_unchecked(index as usize) }
            }
        }
        impl<T> core::ops::IndexMut<$ty> for [T; <$ty>::LEN] {
            #[inline(always)]
            fn index_mut(&mut self, index: $ty) -> &mut T {
                unsafe { self.get_unchecked_mut(index as usize) }
            }
        }
        impl $ty {
            pub const ALL: [Self; Self::LEN] = {
                konst::array::from_fn!(|i| unsafe { Self::from_int_unchecked(i as u8) })
            };
            #[inline(always)]
            pub const fn from_int(int: u8) -> Option<Self> {
                if (int as usize) < Self::LEN { Some(unsafe { core::mem::transmute::<u8, Self>(int) }) } else { None }

            }
            /// # Safety
            /// int must be less than `Self::LEN`
            #[inline(always)]
            pub const unsafe fn from_int_unchecked(int: u8) -> Self {
                unsafe { core::mem::transmute(int) }
            }
        })*
    };
}

impl_index!(Side, PieceKind, Piece, Square, Rank, File);
