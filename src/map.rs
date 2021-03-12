use bracket_lib::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Position {
    pub x: usize,
    pub y: usize
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Tile {
    Road,
}

impl Tile {
    pub fn render(&self, point: &Point, ctx: &mut BTerm) {
        match self {
            Tile::Road => ctx.print_color(point.x, point.y, BLACK, DARK_GREEN, "#"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<Vec<Tile>>,
}

impl Map {
    pub fn fill(width: usize, height: usize, tile: Tile) -> Self {
        Self {
            width,
            height,
            tiles: vec![vec![tile; height]; width],
        }
    }

    pub fn render(&self, ctx: &mut BTerm) {
        for x in 0..self.width {
            for y in 0..self.height {
                let point = Point::new(x, y);
                self.tiles[x][y].render(&point, ctx);
            }
        }
    }
}
