use std::ops::{BitAnd, BitOr, Not, Shl, Shr};

pub trait Buffer:
    Shl<u32, Output = Self>
    + Shr<u32, Output = Self>
    + BitOr<Output = Self>
    + BitAnd<Output = Self>
    + Not<Output = Self>
    + Eq
    + Copy
{
    const ZERO: Self;
    const ONE: Self;
    const MAX: Self;
    const BITS: u32;
}

macro_rules! impl_buffer {
    ($($t:ty),*) => {
        $(
            impl Buffer for $t {
                const ZERO: Self = 0;
                const ONE: Self = 1;
                const MAX: Self = <$t>::MAX;
                const BITS: u32 = <$t>::BITS;
            }
        )*
    };
}

impl_buffer!(u8, u16, u32, u64, u128);
