use crate::{Buffer, Packable, shared::mask};

/// Sequentially packs values into a single buffer.
///
/// Each [`pack`](Self::pack) call shifts the existing buffer left by the new
/// field's `SIZE` and inserts the new value into the freed lower bits, so the
/// first packed field ends up in the highest bits.
///
/// Most callers will not interact with `Packer` directly — the
/// [`#[packable]`](crate::packable) macro generates code that uses it.
///
/// # Example
///
/// ```
/// use bitcram::Packer;
///
/// let mut packer = Packer::<u32>::new();
/// packer.raw_pack(0b101, 3);
/// packer.raw_pack(0b1, 1);
/// assert_eq!(packer.into_inner(), 0b1011);
/// ```
#[derive(Debug)]
pub struct Packer<B> {
    buffer: B,
    #[cfg(debug_assertions)]
    bits_packed: u32,
}

impl<B: Buffer> Packer<B> {
    /// Creates a new packer with a zeroed buffer.
    #[inline]
    pub fn new() -> Self {
        Self {
            buffer: B::ZERO,
            #[cfg(debug_assertions)]
            bits_packed: 0,
        }
    }

    /// Packs `packable` into the buffer using its [`Packable`] implementation.
    #[inline]
    pub fn pack<P: Packable<B>>(&mut self, packable: &P) {
        self.raw_pack(packable.pack(), P::SIZE);
    }

    /// Packs the lower `size` bits of `packed` into the buffer.
    ///
    /// `size` must satisfy `size < B::BITS`. Filling the buffer exactly
    /// requires multiple calls each smaller than `B::BITS`.
    ///
    /// In debug builds, this method asserts that `packed` fits in `size` bits
    /// and that the cumulative packed size does not exceed `B::BITS`. In
    /// release builds, oversized values are silently masked.
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

    /// Consumes the packer and returns the underlying buffer.
    #[inline]
    pub fn into_inner(self) -> B {
        self.buffer
    }
}
