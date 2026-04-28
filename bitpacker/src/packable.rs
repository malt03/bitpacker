use crate::Buffer;

pub trait Packable<B: Buffer> {
    const SIZE: u32;
    fn pack(&self) -> B;
    fn unpack(buffer: B) -> Self;
}
