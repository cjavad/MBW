//! Map generation happens in a series of steps:
//! * Devide map into [`Chunk`]s.
//! * Place [`Structure`]s randomly until no more can be placed.
//! * Generate tiles from those structures.

use crate::map::{self, Tile};
use crate::world::Location;
use rand::prelude::*;

#[derive(Clone, Debug)]
pub struct Chunk {
    pub tiles: [[map::Tile; 6]; 6],
}

#[derive(Clone, Debug)]
pub struct Structure {
    pub chunks: &'static [(map::Position, Chunk)],
}

impl Structure {
    /// Gets the width and height in chunks.
    ///
    /// **NOTE**: this is not particularly efficient, but it should suffice.
    pub fn dimensions(&self) -> (usize, usize) {
        (
            self.chunks.iter().map(|(p, _)| p.x).max().unwrap() + 1,
            self.chunks.iter().map(|(p, _)| p.y).max().unwrap() + 1,
        )
    }
}

/// *width* and *height* is messured in [`Chunk`]s which are 6x6 characters.
#[derive(Clone, Debug)]
pub struct MapGenerationSettings<'a> {
    pub width: usize,
    pub height: usize,
    pub structures: &'a [Structure],
}

impl<'a> MapGenerationSettings<'a> {
    /// Generates the map.
    pub fn generate(&self, rng: &mut impl Rng) -> map::Map {
        let mut chunks: Vec<Vec<Option<(&Chunk, Location)>>> =
            vec![vec![None; self.height]; self.width];

        // because hardcoding is okay if behind atleast 2 levels of indirection
        const MAX_TRIES: u32 = 10;

        // try to place buildings, if 10 tries don't do, then nothing will
        let mut times_tried = 0;
        loop {
            let structure = self.structures.choose(rng).unwrap();
            let (width, height) = structure.dimensions();

            // generate position to try placing
            let x = rng.gen_range(0..=self.width - width);
            let y = rng.gen_range(0..=self.height - height);

            // check if place is available
            let can_place: bool = structure
                .chunks
                .iter()
                .map(|(p, _)| chunks[x + p.x][y + p.y].is_none())
                .fold(true, |a, b| a && b);

            // if can place, then do so
            if can_place {
                let location = Location::generate(rng);

                for (p, chunk) in structure.chunks {
                    chunks[x + p.x][y + p.y] = Some((chunk, location.clone()));
                }

                times_tried = 0;
            } else {
                times_tried += 1;

                if times_tried >= MAX_TRIES {
                    break;
                }
            }
        }

        let mut map = map::Map::fill(self.width * 6, self.height * 6, map::Tile::Empty);

        // replace tiles with the generated ones
        for column in 0..self.width {
            for row in 0..self.height {
                if let Some((chunk, location)) = chunks[column][row].clone() {
                    let color = location.color();

                    for x in 0..6 {
                        for y in 0..6 {
                            map.tiles[column * 6 + x][row * 6 + y] = chunk.tiles[x][y].clone();

                            match &mut map.tiles[column * 6 + x][row * 6 + y] {
                                Tile::Building(bcolor) => *bcolor = color.clone(),
                                Tile::Door(blocation, _) => *blocation = location.clone(),
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        map
    }
}
