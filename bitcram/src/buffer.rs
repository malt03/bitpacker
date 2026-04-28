use std::ops::{BitAnd, BitOr, Not, Shl, Shr};

/// An unsigned integer type that can serve as a bit-packing buffer.
///
/// Implemented for [`u8`], [`u16`], [`u32`], [`u64`], and [`u128`].
pub trait Buffer:
    Shl<u32, Output = Self>
    + Shr<u32, Output = Self>
    + BitOr<Output = Self>
    + BitAnd<Output = Self>
    + Not<Output = Self>
    + Eq
    + Copy
{
    /// The zero value.
    const ZERO: Self;
    /// The unit value (`1`).
    const ONE: Self;
    /// The maximum value (all bits set).
    const MAX: Self;
    /// The number of bits in this type.
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
