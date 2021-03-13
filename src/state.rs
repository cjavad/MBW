use crate::client::ClientNetworkHandle;
use crate::map::Position;
use crate::map_generation;
use crate::person::{PersonId, PersonUpdate};
use crate::server::WorldUpdate;
use crate::structures;
use crate::ui::Ui;
use crate::world::World;
use bracket_lib::prelude::*;
use std::collections::HashMap;

pub struct State {
    pub width: usize,
    pub height: usize,
    pub world: World,
    pub handle: ClientNetworkHandle,
    pub selected_person: Option<PersonId>,
    pub person_locations: HashMap<Position, Vec<PersonId>>,
}

impl State {
    pub fn new(handle: ClientNetworkHandle) -> Self {
        Self {
            width: crate::MAP_WIDTH_CHUNKS * 6,
            height: crate::MAP_HEIGHT_CHUNKS * 6,
            world: World::empty(crate::MAP_WIDTH_CHUNKS, crate::MAP_HEIGHT_CHUNKS),
            handle,
            selected_person: None,
            person_locations: HashMap::new(),
        }
    }

    pub fn update_person_locations(&mut self) {
        self.person_locations.clear();

        for (id, person) in &self.world.people {
            self.person_locations
                .entry(person.position.clone())
                .or_insert(Vec::new())
                .push(id.clone());
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
        self.update_person_locations();
        self.world.render(ctx, &self.person_locations, Point::new(0, 0));

        let mut ui = Ui::new(ctx, self.width as i32, self.height as i32);

        if ui.left_click {
            if let Some(persons) = self.person_locations.get(&Position::new(
                ui.ctx.mouse_point().x as usize,
                ui.ctx.mouse_point().y as usize,
            )) {
                self.selected_person = Some(persons[0].clone());
            } else {
                self.selected_person = None;
            }
        }

        if let Some(id) = &self.selected_person {
            let person = self.world.people.get(id).unwrap();

            ui.rect(25, 30, |ui| {
                ui.print("Person: ");
                ui.add_offset(Point::new(1, 1));
                ui.print(format!("Alive: {}", person.alive));
                ui.print(format!("Age: {}", person.age));
                ui.print(format!("Job: {}", person.job.ty.as_str()));
                ui.print(format!(
                    "Sex: {}",
                    match person.sex {
                        true => "Male",
                        false => "Female",
                    }
                ));
                ui.print(format!(""));

                ui.add_offset(Point::new(0, 1));

                if person.sick {
                    ui.print("INFECTED!!!");
                }
            });
        }
    }
}
