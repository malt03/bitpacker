use crate::Buffer;

pub(crate) fn mask<B: Buffer>(size: u32) -> B {
    debug_assert!(size <= B::BITS);
    B::MAX >> (B::BITS - size)
}
