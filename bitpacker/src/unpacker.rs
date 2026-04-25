use crate::{Buffer, Packable};

pub struct Unpacker<B> {
    buffer: B,
}

impl<B: Buffer> Unpacker<B> {
    pub fn new(buffer: B) -> Self {
        Self { buffer }
    }

    pub fn unpack<P: Packable<B>>(&mut self) -> P {
        debug_assert!(P::SIZE <= B::BITS);
        let mask = B::MAX >> (B::BITS - P::SIZE);
        let packed = self.buffer & mask;
        self.buffer = self.buffer >> P::SIZE;
        P::unpack(packed)
    }

    pub fn into_inner(self) -> B {
        self.buffer
    }
}
