use bitcram::{Packable, packable};

#[derive(Debug, PartialEq, Eq)]
struct Coord {
    x: u8,
    y: u8,
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
    King,
    Queen,
    Rook,
    Bishop,
    Knight,
    Pawn,
}

#[packable(u16)]
#[derive(Debug, PartialEq, Eq)]
enum Promotion {
    None,
    Queen,
    Rook,
    Bishop,
    Knight,
}

#[packable(u16)]
#[derive(Debug, PartialEq, Eq)]
struct Move {
    from: Coord,
    to: Coord,
    piece: Piece,
    promotion: Promotion,
}

fn main() {
    let m = Move {
        from: Coord { x: 0, y: 1 },
        to: Coord { x: 0, y: 3 },
        piece: Piece::Pawn,
        promotion: Promotion::None,
    };
    let packed: u16 = m.pack();
    assert_eq!(Move::unpack(packed), m);
    println!(
        "Move encodes to {} bits, packed value: {:#018b}",
        Move::SIZE,
        packed
    );
    println!("Round-trip OK");
}
