use bracket_lib::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub const fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum Tile {
    Empty,
    Building,
    Door(Option<u32>),
    RoadBlock,
    TestCenter,
    VaccineCenter,
    MaskCampain(u32),
    AntivaxCampain(u32),
}

impl Tile {
    pub fn render(&self, point: &Point, ctx: &mut BTerm) {
        match self {
            Tile::Empty => {}
            Tile::Building => ctx.print_color(point.x, point.y, DARKOLIVEGREEN4, BLACK, "#"),
            Tile::Door(lockdown) => ctx.print_color(point.x, point.y, BURLYWOOD, BLACK, "["),
            Tile::RoadBlock => ctx.print_color(point.x, point.y, GRAY, BLACK, "X"),
            Tile::TestCenter => ctx.print_color(point.x, point.y, PURPLE, BLACK, "T"),
            Tile::VaccineCenter => ctx.print_color(point.x, point.y, GREEN, BLACK, "V"),
            Tile::MaskCampain(_) => ctx.print_color(point.x, point.y, GOLD, BLACK, "M"),
            Tile::AntivaxCampain(_) => ctx.print_color(point.x, point.y, RED, BLACK, "A"),
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

    pub fn get_tile(&self, position: &Position) -> &Tile {
        &self.tiles[position.x][position.y]
    }

    pub fn render(&self, ctx: &mut BTerm, offset: Point) {
        for x in 0..self.width {
            for y in 0..self.height {
                let point = Point::new(x, y) + offset;
                self.tiles[x][y].render(&point, ctx);
            }
        }
    }

    pub fn in_bounds(&self, position: &Position) -> bool {
        position.x < self.width && position.y < self.height
    }

    pub fn can_walk(&self, position: &Position) -> bool {
        if self.in_bounds(position) {
            match self.tiles[position.x][position.y] {
                Tile::Empty => true,
                Tile::Door(lockdown) => lockdown.is_none(),
                Tile::TestCenter => true,
                Tile::VaccineCenter => true,
                _ => false,
            }
        } else {
            false
        }
    }
}
