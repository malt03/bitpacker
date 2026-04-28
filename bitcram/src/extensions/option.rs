use crate::{Buffer, Packable, Packer, Unpacker};

impl<B: Buffer, T: Packable<B>> Packable<B> for Option<T> {
    const SIZE: u32 = 1 + T::SIZE;

    fn pack(&self) -> B {
        let mut packer = Packer::<B>::new();
        match self {
            None => packer.pack(&false),
            Some(value) => {
                packer.pack(value);
                packer.pack(&true);
            }
        }
        packer.into_inner()
    }

    fn unpack(buffer: B) -> Self {
        let mut unpacker = Unpacker::<B>::new(buffer);
        if unpacker.unpack() {
            Some(unpacker.unpack())
        } else {
            None
        }
    }
}
