use crate::Buffer;

/// A type that can be packed into and unpacked from a [`Buffer`] of type `B`.
///
/// The [`#[packable(B)]`](crate::packable) attribute macro derives this trait
/// for structs and enums automatically. Implement it manually for primitive
/// types or for custom encodings.
///
/// # Manual implementation contract
///
/// - [`pack`](Self::pack) must return a value where bits at and above position
///   `SIZE` are zero. Debug builds verify this with `assert!`; release builds
///   silently mask oversized values.
/// - [`unpack`](Self::unpack) receives a value where only the lower `SIZE` bits
///   are meaningful; higher bits are not guaranteed to be zero.
///
/// # Example
///
/// A 5×6 board coordinate packed into 5 bits — fewer than the 6 bits a naive
/// "3 bits for x + 3 bits for y" layout would use:
///
/// ```
/// use bitpacker::Packable;
///
/// struct Coord { x: u8, y: u8 }
///
/// impl Packable<u16> for Coord {
///     const SIZE: u32 = 5;
///     fn pack(&self) -> u16 {
///         (self.y * 5 + self.x) as u16
///     }
///     fn unpack(buffer: u16) -> Self {
///         let i = buffer as u8;
///         Self { x: i % 5, y: i / 5 }
///     }
/// }
/// ```
pub trait Packable<B: Buffer> {
    /// The number of bits this type occupies when packed.
    const SIZE: u32;

    /// Packs `self` into the lower `SIZE` bits of a buffer.
    fn pack(&self) -> B;

    /// Reconstructs a value from the lower `SIZE` bits of `buffer`.
    fn unpack(buffer: B) -> Self;
}
