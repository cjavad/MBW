use crate::client::ClientNetworkHandle;
use crate::map::Position;
use crate::person::{PersonId, PersonUpdate};
use crate::server::WorldUpdate;
use crate::ui::{DrawContext, DrawUi, Rect, Ui};
use crate::world::World;
use bracket_lib::prelude::*;
use std::collections::HashMap;

pub struct State {
    pub side: bool,
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
            side: true,
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
            self.world.time.set_minutes(payload.tick_count as u32);

            println!(
                "days: {}, hours: {}, min: {}",
                self.world.time.days, self.world.time.hours, self.world.time.minutes
            );

            // TODO: networking stuff with time and stuff

            for update in payload.updates {
                // TODO: maybe switch to more broad update type that PersonUpdate

                match update {
                    WorldUpdate::PersonUpdate(person_update) => match person_update {
                        PersonUpdate::Position(id, new_position) => {
                            self.world.people.get_mut(&id).unwrap().position = new_position;
                        }
                        PersonUpdate::Infected(id, is_infected) => {
                            self.world.people.get_mut(&id).unwrap().infected = is_infected;
                        }
                    },
                    WorldUpdate::SetWorld(new_world) => self.world = new_world,
                }
            }
        }
    }

    pub fn virus_ui(&mut self, ui: &mut Ui) {

    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(BLACK);

        self.handle_payloads();
        self.update_person_locations();
        self.world
            .render(ctx, &self.person_locations, Point::new(0, 0));

        let mut ui = Ui::new(
            ctx,
            Rect {
                position: Point::new(0, 0),
                width: self.width as i32,
                height: self.height as i32,
            },
        );

        if self.side {
            self.virus_ui(&mut ui);
        } else {
            
        }

        let selected_person = &mut self.selected_person;
        let world = &self.world;

        if selected_person.is_some() {
            let person = self
                .world
                .people
                .get(selected_person.as_ref().unwrap())
                .unwrap();

            ui.rect(25, 30, |ui| {
                if ui.mouse_click && !ui.clicked() {
                    *selected_person = None;
                }

                ui.print("Person: ");
                ui.offset(Point::new(1, 1));
                ui.print(format!("Name: {} {}", person.first_name, person.last_name));
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

                if person.infected {
                    ui.offset(Point::new(0, 1));
                    ui.print("INFECTED!!!");
                }

                ui.offset(Point::new(0, 1));
                ui.print("Acquaintances:");
                ui.offset(Point::new(1, 1));

                for aq in &person.habits.acquaintances {
                    ui.text(
                        format!(
                            "{} {}",
                            world.people[aq].first_name, world.people[aq].last_name
                        ),
                        |ui| {
                            if ui.clicked() {
                                *selected_person = Some(aq.clone());
                            }
                        },
                    );
                }
            });
        }

        if ui.mouse_click && self.selected_person.is_none() {
            if let Some(persons) = self.person_locations.get(&Position::new(
                ctx.mouse_point().x as usize,
                ctx.mouse_point().y as usize,
            )) {
                self.selected_person = Some(persons[0].clone());
            }
        }

        let mut ctx = DrawContext { bterm: ctx };
        ui.draw(&mut ctx);
    }
}
