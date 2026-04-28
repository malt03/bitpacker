use crate::{Buffer, Packable};

impl<B: Buffer> Packable<B> for bool {
    const SIZE: u32 = 1;

    #[inline]
    fn pack(&self) -> B {
        if *self { B::ONE } else { B::ZERO }
    }

    #[inline]
    fn unpack(buffer: B) -> Self {
        buffer == B::ONE
    }
}
