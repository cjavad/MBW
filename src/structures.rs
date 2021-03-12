use crate::map::{Position, Tile};
use crate::map_generation::*;

pub const STRUCTURES: &[Structure] = &[Structure {
    chunks: &[
        (
            Position::new(0, 0),
            Chunk {
                tiles: [
                    [
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                    ],
                    [
                        Tile::Empty,
                        Tile::Building,
                        Tile::Building,
                        Tile::Door,
                        Tile::Building,
                        Tile::Building,
                    ],
                    [
                        Tile::Empty,
                        Tile::Building,
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                    ],
                    [
                        Tile::Empty,
                        Tile::Building,
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                    ],
                    [
                        Tile::Empty,
                        Tile::Building,
                        Tile::Building,
                        Tile::Building,
                        Tile::Building,
                        Tile::Building,
                    ],
                    [
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                    ],
                ],
            },
        ),
        (
            Position::new(0, 1),
            Chunk {
                tiles: [
                    [
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                    ],
                    [
                        Tile::Building,
                        Tile::Building,
                        Tile::Building,
                        Tile::Building,
                        Tile::Building,
                        Tile::Empty,
                    ],
                    [
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Building,
                        Tile::Empty,
                    ],
                    [
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Building,
                        Tile::Empty,
                    ],
                    [
                        Tile::Building,
                        Tile::Building,
                        Tile::Building,
                        Tile::Building,
                        Tile::Building,
                        Tile::Empty,
                    ],
                    [
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                        Tile::Empty,
                    ],
                ],
            },
        ),
    ],
}];
