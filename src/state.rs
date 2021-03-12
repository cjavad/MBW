use crate::map;
use bracket_lib::prelude::*;
use crate::map_generation;
use crate::structures;

pub struct State {
    pub map: map::Map,
}

impl State {
    pub fn new() -> Self {
        let settings = map_generation::MapGenerationSettings {
            width: 24,
            height: 16,
            structures: structures::STRUCTURES,
        };

        let mut rng = rand::thread_rng();

        Self {
            map: settings.generate(&mut rng),
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(BLACK);

        self.map.render(ctx);
    }
}
