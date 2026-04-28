use std::mem::MaybeUninit;

use crate::{Buffer, Packable, Packer, Unpacker};

impl<B: Buffer, T: Packable<B>, const N: usize> Packable<B> for [T; N] {
    const SIZE: u32 = T::SIZE * N as u32;

    fn pack(&self) -> B {
        let mut packer = Packer::<B>::new();
        for item in self.iter() {
            packer.pack(item);
        }
        packer.into_inner()
    }

    fn unpack(buffer: B) -> Self {
        let mut unpacker = Unpacker::<B>::new(buffer);
        let mut array = [const { MaybeUninit::<T>::uninit() }; N];
        for item in array.iter_mut().rev() {
            item.write(unpacker.unpack());
        }
        unsafe { (&array as *const _ as *const [T; N]).read() }
    }
}
