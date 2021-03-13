use crate::client::ClientNetworkHandle;
use crate::map_generation;
use crate::structures;
use crate::world;
use bracket_lib::prelude::*;

pub struct State {
    pub world: world::World,
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
            world: world::World::generate(settings, &mut rng),
            handle,
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(BLACK);

        self.world.render(ctx, Point::new(0, 0));
    }
}
