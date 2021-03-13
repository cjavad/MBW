use crate::client::ClientNetworkHandle;
use crate::map_generation;
use crate::person::PersonUpdate;
use crate::server::WorldUpdate;
use crate::structures;
use crate::world::World;
use crate::ui::Ui;
use bracket_lib::prelude::*;

pub struct State {
    pub width: usize,
    pub height: usize,
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
            width: crate::MAP_WIDTH_CHUNKS * 6,
            height: crate::MAP_HEIGHT_CHUNKS * 6,
            world: World::empty(crate::MAP_WIDTH_CHUNKS, crate::MAP_HEIGHT_CHUNKS),
            handle,
        }
    }

    pub fn handle_payloads(&mut self) {
        for payload in self.handle.get_payloads() {
            self.world.set_time(payload.age);

            println!(
                "days: {}, hours: {}, min: {}",
                self.world.days, self.world.hours, self.world.minutes
            );

            // TODO: networking stuff with time and stuff

            for update in payload.updates {
                // TODO: maybe switch to more broad update type that PersonUpdate

                match update {
                    WorldUpdate::PersonUpdate(person_update) => match person_update {
                        PersonUpdate::Position(id, new_position) => {
                            self.world.people.get_mut(&id).unwrap().position = new_position;
                        }
                    },
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

        let mut ui = Ui::new(ctx, self.width as i32, self.height as i32);

        ui.add_offset(Point::new(10, 10));
        ui.rect(20, 30, |ui| {
            ui.print("Test menu");
            ui.add_offset(Point::new(1, 1));
            ui.print("Woo, menu shit");
            
            if ui.clicked() {
                println!("i am clicked");
            }
        });
    }
}
