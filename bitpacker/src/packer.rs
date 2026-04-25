use crate::{Buffer, Packable};

pub struct Packer<B> {
    buffer: B,
}

impl<B: Buffer> Packer<B> {
    pub fn new() -> Self {
        Self { buffer: B::ZERO }
    }

    pub fn pack<P: Packable<B>>(&mut self, packable: &P) {
        debug_assert!(P::SIZE <= B::BITS);
        let packed = packable.pack();
        self.buffer = packed | (self.buffer << P::SIZE);
    }

    pub fn into_inner(self) -> B {
        self.buffer
    }
}
