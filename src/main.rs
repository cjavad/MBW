mod map;

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

fn main() -> Result<(), Box<dyn std::error::Error + 'static + Send + Sync>> {
    let ctx = BTermBuilder::simple(200, 100)?.with_title("MBW").with_vsync(true).with_fps_cap(60.0).build()?;

    let state = State::new();

    main_loop(ctx, state)
}