use crate::client::{ClientNetworkHandle, PlayerCommandHandle};
use crate::map::Position;
use crate::person::{PersonId, PersonUpdate};
use crate::server::{PlayerCommand, StateUpdate};
use crate::ui::{DrawContext, DrawUi, Rect, Ui};
use crate::world::World;
use bracket_lib::prelude::*;
use rand::prelude::*;
use std::collections::HashMap;

pub enum Ability {
    AntivaxCampain,
    Roadblock,
    SocialImpulse,
    Testcenter,
    Lockdown,
    Vaccinecenter,
    MaskCampain,
}

impl Ability {
    pub fn as_str(&self) -> &str {
        match self {
            Ability::AntivaxCampain => "Antivax Campaign",
            Ability::Roadblock => "Roadblock",
            Ability::SocialImpulse => "Social Impulse",
            Ability::Testcenter => "Testcenter",
            Ability::Vaccinecenter => "Vaccinecenter",
            Ability::Lockdown => "Lockdown",
            Ability::MaskCampain => "Mask Campaign",
        }
    }
}

pub struct State {
    pub side: bool,
    pub width: usize,
    pub height: usize,
    pub money: u32,
    pub world: World,
    pub handle: ClientNetworkHandle,
    pub command_handle: PlayerCommandHandle,
    pub selected_person: Option<PersonId>,
    pub selected_ability: Option<Ability>,
    pub person_locations: HashMap<Position, Vec<PersonId>>,
}

impl State {
    pub fn new(handle: ClientNetworkHandle, command_handle: PlayerCommandHandle) -> Self {
        Self {
            side: true,
            width: crate::MAP_WIDTH_CHUNKS * 6,
            height: crate::MAP_HEIGHT_CHUNKS * 6,
            money: 0,
            world: World::empty(crate::MAP_WIDTH_CHUNKS, crate::MAP_HEIGHT_CHUNKS),
            handle,
            command_handle,
            selected_person: None,
            selected_ability: None,
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

            self.side = payload.side;
            self.money = payload.money;

            // TODO: networking stuff with time and stuff

            for update in payload.updates {
                // TODO: maybe switch to more broad update type that PersonUpdate

                match update {
                    StateUpdate::PersonUpdate(person_update) => match person_update {
                        PersonUpdate::Position(id, new_position) => {
                            self.world.people.get_mut(&id).unwrap().position = new_position;
                        }
                        PersonUpdate::Infected(id, is_infected) => {
                            self.world.people.get_mut(&id).unwrap().infected = is_infected;
                        }
                        PersonUpdate::LifeStatus(id, is_alive) => {
                            self.world.people.get_mut(&id).unwrap().alive = is_alive;
                        }
                        PersonUpdate::Habits(id, habits) => {
                            self.world.people.get_mut(&id).unwrap().habits = habits;
                        }
                        PersonUpdate::Tested(id, tested) => {
                            self.world.people.get_mut(&id).unwrap().tested = tested;
                        }
                        PersonUpdate::Vaccinated(id, vaccinated) => {
                            self.world.people.get_mut(&id).unwrap().vaccinated = vaccinated;
                        }
                    },
                    StateUpdate::TileUpdate(position, tile) => {
                        self.world.map.tiles[position.x][position.y] = tile;
                    }
                    StateUpdate::SetWorld(new_world) => self.world = new_world,
                    StateUpdate::Winner(winner) => { println!("{} won", winner); panic!(); },
                }
            }
        }
    }

    const ABILITY_RECT_WIDTH: i32 = 20;

    pub fn virus_ui(&mut self, ui: &mut Ui) {
        ui.print("VIRUS:");
        ui.print(" Your job is to");
        ui.print(" infect the city.");

        ui.offset(Point::new(0, 1));
        ui.print("Abilities:");

        ui.offset(Point::new(1, 1));
        ui.rect(Self::ABILITY_RECT_WIDTH, 6, |ui| {
            ui.offset(Point::new(1, 1));
            ui.print("Roadblock");
            ui.print(format!(
                "Cost: {}",
                PlayerCommand::Roadblock(Default::default()).price_lookup(self.side)
            ));

            if ui.clicked() {
                self.selected_ability = Some(Ability::Roadblock);
            }
        });

        ui.offset(Point::new(0, 1));
        ui.rect(Self::ABILITY_RECT_WIDTH, 6, |ui| {
            ui.offset(Point::new(1, 1));
            ui.print("Party Impulse");
            ui.print(format!(
                "Cost: {}",
                PlayerCommand::PartyImpulse(Default::default()).price_lookup(self.side)
            ));

            if ui.clicked() {
                if let Some(selected_person) = &self.selected_person {
                    self.command_handle
                        .send(PlayerCommand::PartyImpulse(selected_person.clone()));
                }
            }
        });

        ui.offset(Point::new(0, 1));
        ui.rect(Self::ABILITY_RECT_WIDTH, 6, |ui| {
            ui.offset(Point::new(1, 1));
            ui.print("Social Impulse");
            ui.print(format!(
                "Cost: {}",
                PlayerCommand::SocialImpulse(Default::default()).price_lookup(self.side)
            ));

            if ui.clicked() {
                self.selected_ability = Some(Ability::SocialImpulse);
            }
        });

        ui.offset(Point::new(0, 1));
        ui.rect(Self::ABILITY_RECT_WIDTH, 6, |ui| {
            ui.offset(Point::new(1, 1));
            ui.print("Economic Crash");
            ui.print(format!(
                "Cost: {}",
                PlayerCommand::EconomicCrash.price_lookup(self.side)
            ));

            if ui.clicked() {
                self.command_handle.send(PlayerCommand::EconomicCrash);
            }
        });
    }

    pub fn president_ui(&mut self, ui: &mut Ui) {
        ui.print("PRECIDENT:");
        ui.print(" Your job is to keep");
        ui.print(" the city from being");
        ui.print(" INFECTED.");

        ui.offset(Point::new(0, 1));
        ui.print("Abilities:");

        ui.offset(Point::new(1, 1));
        ui.rect(Self::ABILITY_RECT_WIDTH, 6, |ui| {
            ui.offset(Point::new(1, 1));
            ui.print("Roadblock");
            ui.print(format!(
                "Cost: {}",
                PlayerCommand::Roadblock(Default::default()).price_lookup(self.side)
            ));

            if ui.clicked() {
                self.selected_ability = Some(Ability::Roadblock);
            }
        });

        ui.offset(Point::new(0, 1));
        ui.rect(Self::ABILITY_RECT_WIDTH, 6, |ui| {
            ui.offset(Point::new(1, 1));
            ui.print("Mask Campaign");
            ui.print(format!(
                "Cost: {}",
                PlayerCommand::MaskCampaign(Default::default()).price_lookup(self.side)
            ));

            if ui.clicked() {
                self.selected_ability = Some(Ability::MaskCampain);
            }
        });

        ui.offset(Point::new(0, 1));
        ui.rect(Self::ABILITY_RECT_WIDTH, 6, |ui| {
            ui.offset(Point::new(1, 1));
            ui.print("Lockdown");
            ui.print(format!(
                "Cost: {}",
                PlayerCommand::Lockdown(Default::default()).price_lookup(self.side)
            ));

            if ui.clicked() {
                self.selected_ability = Some(Ability::Lockdown);
            }
        });

        ui.offset(Point::new(0, 1));
        ui.rect(Self::ABILITY_RECT_WIDTH, 6, |ui| {
            ui.offset(Point::new(1, 1));
            ui.print("Testcenter");
            ui.print(format!(
                "Cost: {}",
                PlayerCommand::Testcenter(Default::default()).price_lookup(self.side)
            ));

            if ui.clicked() {
                self.selected_ability = Some(Ability::Testcenter);
            }
        });

        ui.offset(Point::new(0, 1));
        ui.rect(Self::ABILITY_RECT_WIDTH, 6, |ui| {
            ui.offset(Point::new(1, 1));
            ui.print("Vaccinecenter");
            ui.print(format!(
                "Cost: {}",
                PlayerCommand::Vaccinecenter(Default::default()).price_lookup(self.side)
            ));

            if ui.clicked() {
                self.selected_ability = Some(Ability::Vaccinecenter);
            }
        });
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls_bg(BLACK);

        self.handle_payloads();
        self.update_person_locations();
        self.world
            .render(ctx, &self.person_locations, Point::new(30, 0), self.side);

        let mut rng = rand::thread_rng();

        let mut ui = Ui::new(
            ctx,
            Rect {
                position: Point::new(0, 0),
                width: self.width as i32,
                height: self.height as i32,
            },
        );

        ui.rect(30, self.height as i32, |ui| {
            ui.offset(Point::new(1, 1));

            ui.print(format!(
                "Time: {:#02}:{:#02}:{:#02}",
                self.world.time.days, self.world.time.hours, self.world.time.minutes
            ));
            ui.print(format!("Money: {}$", self.money,));

            ui.offset(Point::new(0, 1));

            if self.side {
                self.virus_ui(ui);
            } else {
                self.president_ui(ui);
            }

            if let Some(ability) = &self.selected_ability {
                ui.offset(Point::new(0, 2));
                ui.print("Selected:");
                ui.print(format!(" {}", ability.as_str()));
            }

            ui.offset(Point::new(0, 2));
            ui.print("Color codes:");
            ui.offset(Point::new(1, 1));
            ui.print_color(LIGHT_BLUE, "&: Untested");
            ui.offset(Point::new(0, 1));
            ui.print_color(GREEN2, "&: Tested");
            ui.offset(Point::new(0, 1));
            ui.print_color(BLUE, "&: Vaccinated");
            ui.offset(Point::new(0, 1));
            ui.print_color(ORANGE, "&: Half Infected Group");
            ui.offset(Point::new(0, 1));
            ui.print_color(DARK_RED, "&: Infected");
        });

        ui.set_offset(Point::new(30, 0));

        let selected_person = &mut self.selected_person;
        let world = &self.world;

        if selected_person.is_some() {
            let person = self
                .world
                .people
                .get(selected_person.as_ref().unwrap())
                .unwrap();

            ui.rect(30, 40, |ui| {
                if ui.mouse_click && !ui.clicked() {
                    *selected_person = None;
                }

                ui.print("Person: ");
                ui.offset(Point::new(1, 1));
                ui.print(format!("Name: {} {}", person.first_name, person.last_name));
                ui.offset(Point::new(0, 1));
                ui.print(format!("Alive: {}", person.alive));
                ui.offset(Point::new(0, 1));
                ui.print(format!("Age: {}", person.age));
                ui.offset(Point::new(0, 1));
                ui.print(format!("Job: {}", person.job.ty.as_str()));
                ui.offset(Point::new(0, 1));
                ui.print(format!("Employed: {}", person.job.location.is_some()));
                ui.offset(Point::new(0, 1));
                ui.print(format!(
                    "Sex: {}",
                    match person.sex {
                        true => "Male",
                        false => "Female",
                    }
                ));
                ui.offset(Point::new(0, 1));
                ui.print(format!("Tested: {}", person.tested));
                ui.offset(Point::new(0, 1));
                ui.print(format!("Vaccinated: {}", person.vaccinated));
                ui.offset(Point::new(0, 1));

                let wears_mask = match person.habits.mask {
                    n if n < 0.1 => "Never",
                    n if n < 0.3 => "Sometimes",
                    n if n < 0.6 => "Often",
                    n if n < 0.9 => "Usually",
                    _ => "Always",
                };

                ui.print(format!("Wears mask: {}", wears_mask));

                if person.infected {
                    ui.offset(Point::new(0, 1));
                    ui.print_color(DARK_RED, "INFECTED!!!");
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
            if ctx.mouse_point().x >= 30 {
                if let Some(ability) = &self.selected_ability {
                    let position = Position::new(
                        ctx.mouse_point().x as usize - 30,
                        ctx.mouse_point().y as usize,
                    );

                    match ability {
                        Ability::AntivaxCampain => {
                            self.command_handle
                                .send(PlayerCommand::AntivaxCampaign(position));
                        }
                        Ability::Roadblock => {
                            self.command_handle.send(PlayerCommand::Roadblock(position));
                        }
                        Ability::SocialImpulse => {
                            self.command_handle
                                .send(PlayerCommand::SocialImpulse(position));
                        }
                        Ability::Testcenter => {
                            self.command_handle
                                .send(PlayerCommand::Testcenter(position));
                        }
                        Ability::Lockdown => {
                            self.command_handle.send(PlayerCommand::Lockdown(position));
                        }
                        Ability::Vaccinecenter => {
                            self.command_handle
                                .send(PlayerCommand::Vaccinecenter(position));
                        }
                        Ability::MaskCampain => {
                            self.command_handle
                                .send(PlayerCommand::MaskCampaign(position));
                        }
                    }
                } else {
                    if let Some(persons) = self.person_locations.get(&Position::new(
                        ctx.mouse_point().x as usize - 30,
                        ctx.mouse_point().y as usize,
                    )) {
                        if self.side {
                            let infected = persons.iter().filter(|p| self.world.people[p].infected);

                            self.selected_person = Some(
                                infected
                                    .choose(&mut rng)
                                    .unwrap_or(persons.choose(&mut rng).unwrap())
                                    .clone(),
                            );
                        } else {
                            self.selected_person = Some(persons.choose(&mut rng).unwrap().clone());
                        }
                    }
                }

                self.selected_ability = None;
            }
        }

        let mut ctx = DrawContext { bterm: ctx };
        ui.draw(&mut ctx);
    }
}
