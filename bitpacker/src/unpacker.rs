use crate::{Buffer, Packable, shared::mask};

/// Sequentially unpacks values from a buffer.
///
/// Each [`unpack`](Self::unpack) call extracts the lower `SIZE` bits of the
/// buffer and shifts the remaining bits down, so values are extracted in
/// reverse order of how they were packed.
///
/// Most callers will not interact with `Unpacker` directly — the
/// [`#[packable]`](crate::packable) macro generates code that uses it.
///
/// # Example
///
/// ```
/// use bitpacker::Unpacker;
///
/// let mut unpacker = Unpacker::<u32>::new(0b1011);
/// assert_eq!(unpacker.raw_unpack(1), 0b1);
/// assert_eq!(unpacker.raw_unpack(3), 0b101);
/// ```
#[derive(Debug)]
pub struct Unpacker<B> {
    buffer: B,
}

impl<B: Buffer> Unpacker<B> {
    /// Creates a new unpacker over `buffer`.
    #[inline]
    pub fn new(buffer: B) -> Self {
        Self { buffer }
    }

    /// Extracts a value from the buffer using its [`Packable`] implementation.
    #[inline]
    pub fn unpack<P: Packable<B>>(&mut self) -> P {
        P::unpack(self.raw_unpack(P::SIZE))
    }

    /// Extracts the lower `size` bits of the buffer and advances past them.
    ///
    /// `size` must satisfy `size < B::BITS`. Reading exactly `B::BITS` of
    /// data requires multiple calls each smaller than `B::BITS`.
    #[inline]
    pub fn raw_unpack(&mut self, size: u32) -> B {
        if size == 0 {
            return B::ZERO;
        }
        let packed = self.buffer & mask::<B>(size);
        self.buffer = self.buffer >> size;
        packed
    }

    /// Consumes the unpacker and returns the remaining buffer state.
    #[inline]
    pub fn into_inner(self) -> B {
        self.buffer
    }
}
