use bincode::{Decode, Encode};
use bitfield_struct::bitfield;
use bitcram::{Packable, packable};
use criterion::{Criterion, criterion_group, criterion_main};
use modular_bitfield::specifiers::{B4, B8, B16};
use serde::{Deserialize, Serialize};
use std::hint::black_box;

const A: u8 = 5;
const B: u8 = 10;
const C: u8 = 200;
const D: u16 = 40000;

// === bitcram ===

#[derive(Clone, Copy)]
struct U4(u8);
impl Packable<u32> for U4 {
    const SIZE: u32 = 4;
    fn pack(&self) -> u32 {
        self.0 as u32
    }
    fn unpack(buffer: u32) -> Self {
        Self(buffer as u8)
    }
}

#[derive(Clone, Copy)]
struct U8v(u8);
impl Packable<u32> for U8v {
    const SIZE: u32 = 8;
    fn pack(&self) -> u32 {
        self.0 as u32
    }
    fn unpack(buffer: u32) -> Self {
        Self(buffer as u8)
    }
}

#[derive(Clone, Copy)]
struct U16v(u16);
impl Packable<u32> for U16v {
    const SIZE: u32 = 16;
    fn pack(&self) -> u32 {
        self.0 as u32
    }
    fn unpack(buffer: u32) -> Self {
        Self(buffer as u16)
    }
}

#[packable(u32)]
struct BpStruct {
    a: U4,
    b: U4,
    c: U8v,
    d: U16v,
}

// === modular-bitfield ===

#[modular_bitfield::bitfield]
pub struct MbStruct {
    pub a: B4,
    pub b: B4,
    pub c: B8,
    pub d: B16,
}

// === bitfield-struct ===

#[bitfield(u32)]
struct BfStruct {
    #[bits(4)]
    a: u8,
    #[bits(4)]
    b: u8,
    c: u8,
    d: u16,
}

// === byte-level ===

#[derive(Clone, Copy, Encode, Decode, Serialize, Deserialize)]
struct ByteStruct {
    a: u8,
    b: u8,
    c: u8,
    d: u16,
}

fn bench_pack(c: &mut Criterion) {
    let mut group = c.benchmark_group("pack");

    group.bench_function("bitcram", |b| {
        b.iter(|| {
            let s = BpStruct {
                a: U4(black_box(A)),
                b: U4(black_box(B)),
                c: U8v(black_box(C)),
                d: U16v(black_box(D)),
            };
            black_box(s.pack())
        });
    });

    group.bench_function("modular_bitfield", |b| {
        b.iter(|| {
            let mb = MbStruct::new()
                .with_a(black_box(A))
                .with_b(black_box(B))
                .with_c(black_box(C))
                .with_d(black_box(D));
            black_box(mb.into_bytes())
        });
    });

    group.bench_function("bitfield_struct", |b| {
        b.iter(|| {
            let bf = BfStruct::new()
                .with_a(black_box(A))
                .with_b(black_box(B))
                .with_c(black_box(C))
                .with_d(black_box(D));
            black_box(bf.into_bits())
        });
    });

    group.bench_function("bincode", |b| {
        let mut buf = [0u8; 16];
        let config = bincode::config::standard();
        b.iter(|| {
            let val = ByteStruct {
                a: black_box(A),
                b: black_box(B),
                c: black_box(C),
                d: black_box(D),
            };
            let n = bincode::encode_into_slice(&val, &mut buf, config).unwrap();
            black_box(n)
        });
    });

    group.bench_function("postcard", |b| {
        let mut buf = [0u8; 16];
        b.iter(|| {
            let val = ByteStruct {
                a: black_box(A),
                b: black_box(B),
                c: black_box(C),
                d: black_box(D),
            };
            let used = postcard::to_slice(&val, &mut buf).unwrap();
            black_box(used.len())
        });
    });

    group.finish();
}

fn bench_unpack(c: &mut Criterion) {
    let mut group = c.benchmark_group("unpack");

    let bp_packed: u32 = BpStruct {
        a: U4(A),
        b: U4(B),
        c: U8v(C),
        d: U16v(D),
    }
    .pack();

    let mb_bytes: [u8; 4] = MbStruct::new()
        .with_a(A)
        .with_b(B)
        .with_c(C)
        .with_d(D)
        .into_bytes();

    let bf_bits: u32 = BfStruct::new()
        .with_a(A)
        .with_b(B)
        .with_c(C)
        .with_d(D)
        .into_bits();

    let bincode_bytes: Vec<u8> = bincode::encode_to_vec(
        ByteStruct {
            a: A,
            b: B,
            c: C,
            d: D,
        },
        bincode::config::standard(),
    )
    .unwrap();

    let postcard_bytes: Vec<u8> = postcard::to_allocvec(&ByteStruct {
        a: A,
        b: B,
        c: C,
        d: D,
    })
    .unwrap();

    group.bench_function("bitcram", |b| {
        b.iter(|| {
            let s = BpStruct::unpack(black_box(bp_packed));
            black_box((s.a.0, s.b.0, s.c.0, s.d.0))
        });
    });

    group.bench_function("modular_bitfield", |b| {
        b.iter(|| {
            let mb = MbStruct::from_bytes(black_box(mb_bytes));
            black_box((mb.a(), mb.b(), mb.c(), mb.d()))
        });
    });

    group.bench_function("bitfield_struct", |b| {
        b.iter(|| {
            let bf = BfStruct::from_bits(black_box(bf_bits));
            black_box((bf.a(), bf.b(), bf.c(), bf.d()))
        });
    });

    group.bench_function("bincode", |b| {
        let config = bincode::config::standard();
        b.iter(|| {
            let (s, _): (ByteStruct, usize) =
                bincode::decode_from_slice(black_box(&bincode_bytes), config).unwrap();
            black_box((s.a, s.b, s.c, s.d))
        });
    });

    group.bench_function("postcard", |b| {
        b.iter(|| {
            let s: ByteStruct = postcard::from_bytes(black_box(&postcard_bytes)).unwrap();
            black_box((s.a, s.b, s.c, s.d))
        });
    });

    group.finish();
}

fn bench_round_trip(c: &mut Criterion) {
    let mut group = c.benchmark_group("round_trip");

    group.bench_function("bitcram", |b| {
        b.iter(|| {
            let s = BpStruct {
                a: U4(black_box(A)),
                b: U4(black_box(B)),
                c: U8v(black_box(C)),
                d: U16v(black_box(D)),
            };
            let packed = s.pack();
            let unpacked = BpStruct::unpack(black_box(packed));
            black_box((unpacked.a.0, unpacked.b.0, unpacked.c.0, unpacked.d.0))
        });
    });

    group.bench_function("modular_bitfield", |b| {
        b.iter(|| {
            let mb = MbStruct::new()
                .with_a(black_box(A))
                .with_b(black_box(B))
                .with_c(black_box(C))
                .with_d(black_box(D));
            let bytes = mb.into_bytes();
            let mb2 = MbStruct::from_bytes(black_box(bytes));
            black_box((mb2.a(), mb2.b(), mb2.c(), mb2.d()))
        });
    });

    group.bench_function("bitfield_struct", |b| {
        b.iter(|| {
            let bf = BfStruct::new()
                .with_a(black_box(A))
                .with_b(black_box(B))
                .with_c(black_box(C))
                .with_d(black_box(D));
            let bits = bf.into_bits();
            let bf2 = BfStruct::from_bits(black_box(bits));
            black_box((bf2.a(), bf2.b(), bf2.c(), bf2.d()))
        });
    });

    group.bench_function("bincode", |b| {
        let mut buf = [0u8; 16];
        let config = bincode::config::standard();
        b.iter(|| {
            let val = ByteStruct {
                a: black_box(A),
                b: black_box(B),
                c: black_box(C),
                d: black_box(D),
            };
            let n = bincode::encode_into_slice(&val, &mut buf, config).unwrap();
            let (s, _): (ByteStruct, usize) =
                bincode::decode_from_slice(black_box(&buf[..n]), config).unwrap();
            black_box((s.a, s.b, s.c, s.d))
        });
    });

    group.bench_function("postcard", |b| {
        let mut buf = [0u8; 16];
        b.iter(|| {
            let val = ByteStruct {
                a: black_box(A),
                b: black_box(B),
                c: black_box(C),
                d: black_box(D),
            };
            let used = postcard::to_slice(&val, &mut buf).unwrap();
            let s: ByteStruct = postcard::from_bytes(black_box(used)).unwrap();
            black_box((s.a, s.b, s.c, s.d))
        });
    });

    group.finish();
}

criterion_group!(benches, bench_pack, bench_unpack, bench_round_trip);
criterion_main!(benches);
