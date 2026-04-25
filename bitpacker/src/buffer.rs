use std::ops::{BitAnd, BitOr, Shl, Shr};

use crate::Packable;

pub trait Buffer:
    Shl<u32, Output = Self>
    + Shr<u32, Output = Self>
    + BitOr<Output = Self>
    + BitAnd<Output = Self>
    + Copy
{
    const ZERO: Self;
    const ONE: Self;
    const MAX: Self;
    const BITS: u32;
}

macro_rules! impl_buffer {
    ($t:ty) => {
        impl Buffer for $t {
            const ZERO: Self = 0;
            const ONE: Self = 1;
            const MAX: Self = <$t>::MAX;
            const BITS: u32 = <$t>::BITS;
        }
    };
}

macro_rules! impl_packable_chain {
    ($head:ty) => {
        impl_buffer!($head);
    };
    ($head:ty, $($rest:ty),+ $(,)?) => {
        $(
            impl Packable<$rest> for $head {
                const SIZE: u32 = <$head>::BITS;
                fn pack(&self) -> $rest {
                    *self as $rest
                }
                fn unpack(buffer: $rest) -> Self {
                    buffer as $head
                }
            }
        )+
        impl_packable_chain!($($rest),+);
        impl_buffer!($head);
    };
}

impl_packable_chain!(u8, u16, u32, u64, u128);
