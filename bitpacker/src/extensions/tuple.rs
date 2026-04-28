use crate::{Buffer, Packable, Packer, Unpacker};

macro_rules! impl_packable_tuple {
    ($($name:ident),+) => {
        impl<B: Buffer, $($name: Packable<B>),+> Packable<B> for ($($name,)+) {
            const SIZE: u32 = 0 $(+ $name::SIZE)+;

            fn pack(&self) -> B {
                let mut packer = Packer::<B>::new();
                #[allow(non_snake_case)]
                let ($($name,)+) = self;
                $(
                    packer.pack($name);
                )+
                packer.into_inner()
            }

            fn unpack(buffer: B) -> Self {
                let mut unpacker = Unpacker::<B>::new(buffer);
                impl_packable_tuple!(@reverse_unpack unpacker; $($name),+);
                ($($name,)+)
            }
        }
    };

    (@reverse_unpack $unpacker:ident; $name:ident, $($rest:ident),+) => {
        impl_packable_tuple!(@reverse_unpack $unpacker; $($rest),+);
        impl_packable_tuple!(@unpack $unpacker; $name);
    };

    (@reverse_unpack $unpacker:ident; $name:ident) => {
        impl_packable_tuple!(@unpack $unpacker; $name);
    };

    (@unpack $unpacker:ident; $name:ident) => {
        #[allow(non_snake_case)]
        let $name: $name = $unpacker.unpack();
    }
}

macro_rules! impl_packable_tuple_chain {
    ($name:ident, $($rest:ident),+) => {
        impl_packable_tuple!($name, $($rest),+);
        impl_packable_tuple_chain!($($rest),+);
    };
    ($name:ident) => {
        impl_packable_tuple!($name);
    };
}

impl_packable_tuple_chain!(O, P, Q, R, S, T, U, V, W, X, Y, Z);
