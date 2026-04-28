# bitcram

[![Crates.io Version](https://img.shields.io/crates/v/bitcram)](https://crates.io/crates/bitcram)
[![docs.rs](https://img.shields.io/docsrs/bitcram)](https://docs.rs/bitcram)
[![Test](https://github.com/malt03/bitcram/actions/workflows/test.yml/badge.svg?event=release)](https://github.com/malt03/bitcram/actions/workflows/test.yml)

A small, derive-based bit packing library for Rust. Pack structured data into compact integer buffers with sub-byte field granularity.

## Features

- `#[packable(B)]` attribute macro for automatic `Packable` implementations
- Buffer types: `u8`, `u16`, `u32`, `u64`, `u128`
- Supports structs (unit, tuple, named) and enums with mixed variant kinds
- Generics with type parameters (bounds auto-applied)
- Debug-mode safety checks with zero release-build overhead
- Performance comparable to established bit-packing crates

## Quick Start

Encode a mini-chess move (5×6 board, 6 piece types, with promotion) into 16 bits:

```rust
use bitcram::{Packable, packable};

/// A coordinate on a 5×6 board.
///
/// 30 distinct (x, y) pairs fit in 5 bits. A naive "3 bits for x + 3 bits for y"
/// layout would use 6 bits — implementing `Packable` directly lets us collapse
/// the two fields into a single 5-bit index and reclaim that bit.
#[derive(Debug, PartialEq, Eq)]
struct Coord {
    x: u8, // 0..5
    y: u8, // 0..6
}

impl Packable<u16> for Coord {
    const SIZE: u32 = 5;
    fn pack(&self) -> u16 {
        (self.y * 5 + self.x) as u16
    }
    fn unpack(buffer: u16) -> Self {
        let i = buffer as u8;
        Self { x: i % 5, y: i / 5 }
    }
}

#[packable(u16)]
#[derive(Debug, PartialEq, Eq)]
enum Piece {
    King, Queen, Rook, Bishop, Knight, Pawn,
} // 6 variants → 3 bits

#[packable(u16)]
#[derive(Debug, PartialEq, Eq)]
enum Promotion {
    None, Queen, Rook, Bishop, Knight,
} // 5 variants → 3 bits

#[packable(u16)]
#[derive(Debug, PartialEq, Eq)]
struct Move {
    from: Coord,          // 5 bits
    to: Coord,            // 5 bits
    piece: Piece,         // 3 bits
    promotion: Promotion, // 3 bits
} // 16 bits total — fits exactly in u16

let m = Move {
    from: Coord { x: 0, y: 1 },
    to: Coord { x: 0, y: 3 },
    piece: Piece::Pawn,
    promotion: Promotion::None,
};
let packed: u16 = m.pack(); // 2 bytes
assert_eq!(Move::unpack(packed), m);
```

Each move serializes to 2 bytes. A naive byte-aligned encoding of the same data would take 4 bytes or more.

## Supported types

`#[packable(B)]` can derive `Packable<B>` for:

- **Unit structs** (`struct Foo;`) — `SIZE = 0`
- **Tuple structs** (`struct Foo(X, Y);`)
- **Named structs** (`struct Foo { x: X, y: Y }`)
- **Enums** with any combination of variant kinds:
  - Unit variants (`Bar`)
  - Empty tuple variants (`Bar()`)
  - Tuple variants (`Bar(X, Y)`)
  - Named variants (`Bar { x: X, y: Y }`)
- **Generic types** — `Packable<B>` bounds are auto-applied to type parameters

Built-in `Packable` implementations are provided for:

| Type | `SIZE` |
|---|---|
| `bool` | `1` |
| `Option<T>` | `1 + T::SIZE` |
| `(T1, T2, ..., Tn)` (up to 12-tuples) | `T1::SIZE + T2::SIZE + ... + Tn::SIZE` |
| `[T; N]` | `N * T::SIZE` |

For primitive types or custom encodings (like `Coord` above), implement `Packable<B>` manually.

## Conventions

The library trades runtime checks for performance, with a few rules the caller is expected to follow:

- **`Packable::pack()` must return a value that fits in `SIZE` bits.** Debug builds verify this with `assert!`; release builds silently mask oversized values to prevent buffer corruption.
- **Total packed size must not exceed `B::BITS`.** Debug builds track cumulative bits; release builds rely on this convention.
- **A single `raw_pack`/`raw_unpack` call must use `size < B::BITS`.** Compose multiple smaller calls if you need to fill the buffer exactly. (Shifting by the full type width is undefined behavior in Rust.)
- **Empty enums are silently skipped** by the derive macro. This allows incremental development without compile errors on placeholder types.
- **The `Unpacker` trusts its input.** Decoding malformed data may produce garbage but will not panic on its own.

## Benchmarks

Compared against similar crates packing 4 fields (4-bit + 4-bit + 8-bit + 16-bit = 32 bits total):

| Crate | Pack | Unpack | Round-trip | Output size |
|---|---:|---:|---:|---:|
| **bitcram** | 934 ps | 1.61 ns | 2.00 ns | 4 bytes |
| modular-bitfield | 967 ps | 1.54 ns | 1.99 ns | 4 bytes |
| bitfield-struct | 970 ps | 1.55 ns | 2.00 ns | 4 bytes |
| bincode 2.x | 3.82 ns | 4.33 ns | 8.09 ns | ~6 bytes |
| postcard | 6.32 ns | 2.98 ns | 9.16 ns | ~6 bytes |

The three bit-packers are essentially equivalent. Byte-level serializers are ~4× slower on round-trip and produce ~50% larger output for this kind of small, fixed-shape data.

Numbers were measured on a single machine and will vary by hardware; reproduce locally with:

```sh
cargo bench -p bitcram_bench
```

## Workspace structure

- `bitcram/` — runtime crate (`Packable`, `Buffer`, `Packer`, `Unpacker`)
- `bitcram_derive/` — proc-macro crate (`#[packable]`)
- `bitcram_bench/` — comparison benchmarks (not published)

## Requirements

- Rust 1.85+ (edition 2024)

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache License 2.0](LICENSE-APACHE) at your option.
