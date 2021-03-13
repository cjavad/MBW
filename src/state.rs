use crate::client::ClientNetworkHandle;
use crate::map_generation;
use crate::server::WorldUpdate;
use crate::person::PersonUpdate;
use crate::structures;
use crate::world::World;
use bracket_lib::prelude::*;

pub struct State {
    pub world: World,
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
            world: World::empty(24, 16),
            handle,
        }
    }

    pub fn handle_payloads(&mut self) {
        for payload in self.handle.get_payloads() {
            self.world.time = payload.age;

            // TODO: networking stuff with time and stuff

            for update in payload.updates {
                // TODO: maybe switch to more broad update type that PersonUpdate

                match update {
                    WorldUpdate::PersonUpdate(_) => {},
                    WorldUpdate::SetWorld(new_world) => self.world = new_world,
                }
            }
        }
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(BLACK);

        self.handle_payloads();
        self.world.render(ctx, Point::new(0, 0));
    }
}
