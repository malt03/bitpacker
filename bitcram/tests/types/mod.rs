use std::fmt::Debug;

use bitcram::{Packable, packable};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct U3(pub u8);
impl Packable<u128> for U3 {
    const SIZE: u32 = 3;
    fn pack(&self) -> u128 {
        self.0 as u128
    }
    fn unpack(buffer: u128) -> Self {
        Self(buffer as u8)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct U64v(pub u64);
impl Packable<u128> for U64v {
    const SIZE: u32 = 64;
    fn pack(&self) -> u128 {
        self.0 as u128
    }
    fn unpack(buffer: u128) -> Self {
        Self(buffer as u64)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Nibble(pub u8);
impl Packable<u8> for Nibble {
    const SIZE: u32 = 4;
    fn pack(&self) -> u8 {
        self.0
    }
    fn unpack(buffer: u8) -> Self {
        Self(buffer)
    }
}

#[packable(u128)]
#[derive(Debug, PartialEq, Eq)]
pub struct UnitStruct;

#[packable(u128)]
#[derive(Debug, PartialEq, Eq)]
pub struct TupleStruct(pub U3, pub U3, pub UnitStruct);

#[packable(u128)]
#[derive(Debug, PartialEq, Eq)]
pub struct NamedStruct {
    pub x: U3,
    pub y: U3,
}

#[packable(u128)]
#[derive(Debug, PartialEq, Eq)]
pub enum MixedEnum {
    UnitVariant,
    EmptyTuple(),
    Tuple(U3, TupleStruct),
    Named { x: UnitStruct, y: TupleStruct },
}

#[packable(u128)]
#[derive(Debug, PartialEq, Eq)]
pub enum SingleVariantEnum {
    Only,
}

#[packable(u128)]
#[derive(Debug, PartialEq, Eq)]
pub struct GenericPair<T: Clone>
where
    T: Debug,
{
    pub x: T,
    pub y: T,
}

#[packable(u128)]
#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum EmptyEnum {}

#[packable(u8)]
#[derive(Debug, PartialEq, Eq)]
pub struct FullU8 {
    pub hi: Nibble,
    pub lo: Nibble,
}

#[packable(u128)]
#[derive(Debug, PartialEq, Eq)]
pub struct FullU128 {
    pub hi: U64v,
    pub lo: U64v,
}

#[packable(u32)]
#[derive(Debug, PartialEq, Eq)]
pub struct BitsStruct {
    #[bits(5)]
    pub x: u8,
    #[bits(5)]
    pub y: u8,
    #[bits(3)]
    pub z: u8,
}

#[packable(u16)]
#[derive(Debug, PartialEq, Eq)]
pub struct BitsTuple(#[bits(4)] pub u8, #[bits(4)] pub u8);

#[packable(u32)]
#[derive(Debug, PartialEq, Eq)]
pub enum BitsEnum {
    Empty,
    Value(#[bits(8)] u8),
    Pair {
        #[bits(4)]
        a: u8,
        b: bool,
    },
}
