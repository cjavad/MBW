use crate::map;
use bracket_lib::prelude::*;

pub struct State {
    pub map: map::Map,
}

impl State {
    pub fn new() -> Self {
        Self {
            map: map::Map::fill(200, 100, map::Tile::Road),
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(BLACK);

        self.map.render(ctx);
    }
}
