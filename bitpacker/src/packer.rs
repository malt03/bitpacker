use crate::{Buffer, Packable, shared::mask};

#[derive(Debug)]
pub struct Packer<B> {
    buffer: B,
    #[cfg(debug_assertions)]
    bits_packed: u32,
}

impl<B: Buffer> Packer<B> {
    #[inline]
    pub fn new() -> Self {
        Self {
            buffer: B::ZERO,
            #[cfg(debug_assertions)]
            bits_packed: 0,
        }
    }

    #[inline]
    pub fn pack<P: Packable<B>>(&mut self, packable: &P) {
        self.raw_pack(packable.pack(), P::SIZE);
    }

    #[inline]
    pub fn raw_pack(&mut self, packed: B, size: u32) {
        if size == 0 {
            return;
        }
        let mask = mask::<B>(size);
        #[cfg(debug_assertions)]
        {
            assert!(
                packed & !mask == B::ZERO,
                "Packed value exceeds the size limit"
            );
            self.bits_packed += size;
            assert!(
                self.bits_packed <= B::BITS,
                "Packed bits exceed buffer size"
            );
        }
        self.buffer = (packed & mask) | (self.buffer << size);
    }

    #[inline]
    pub fn into_inner(self) -> B {
        self.buffer
    }
}
