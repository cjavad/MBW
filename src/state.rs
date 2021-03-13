use crate::map;
use crate::map_generation;
use crate::structures;
use crate::client::ClientNetworkHandle;
use bracket_lib::prelude::*;

pub struct State {
    pub map: map::Map,
    pub handle: ClientNetworkHandle,
}

impl State {
    pub fn new(handle: ClientNetworkHandle) -> Self {
        let settings = map_generation::MapGenerationSettings {
            width: 24,
            height: 16,
            structures: structures::STRUCTURES,
        };

        let mut rng = rand::thread_rng();

        Self {
            map: settings.generate(&mut rng),
            handle,
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(BLACK);

        self.map.render(ctx, Point::new(0, 0));
    }
}
