pub trait Packable<Buffer> {
    const SIZE: u32;
    fn pack(&self) -> Buffer;
    fn unpack(buffer: Buffer) -> Self;
}
