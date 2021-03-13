use crate::map::{Map, Position};
use crate::server::{GameSession, PathCache};
use crate::world::World;
use bracket_lib::prelude::*;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ops::Range;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Job {
    Doctor,
    Programmer,
    Clerk,
    PoliceOfficer,
    FireFighter,
    PublicServant,
    Chef,
    Teacher,
    Student,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PersonUpdate {
    Position(PersonId, Position),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PersonId(pub u32);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PersonAction {
    Working,
    Walking(Vec<Position>, Box<PersonAction>),
    AtHome,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonHabits {
    pub mask: f32,
    pub hygiene: f32,
    pub acquaintances: HashSet<PersonId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Person {
    pub alive: bool,
    pub sick: bool,
    pub age: u8,
    pub sex: bool,
    pub job_type: Job,
    pub job_location: Option<Position>,
    pub position: Position,
    pub home: Position,
    pub habits: PersonHabits,
}

impl Job {
    pub fn infection_rate(&self) -> f32 {
        match self {
            Job::Doctor => 0.5,
            Job::Programmer => 0.5,
            Job::Clerk => 0.5,
            Job::PoliceOfficer => 0.5,
            Job::FireFighter => 0.5,
            Job::PublicServant => 0.5,
            Job::Chef => 0.1,
            Job::Teacher => 0.8,
            Job::Student => 0.0,
        }
    }

    pub fn work_hours(&self) -> Range<u8> {
        match self {
            Job::Doctor => 9..17,
            Job::Programmer => 12..22,
            Job::Clerk => 7..18,
            Job::PoliceOfficer => 6..16,
            Job::FireFighter => 11..21,
            Job::PublicServant => 11..23,
            Job::Chef => 14..23,
            Job::Teacher => 8..17,
            Job::Student => 8..16,
        }
    }

    pub fn generate(rng: &mut impl Rng) -> Self {
        match rng.gen_range(0..9) {
            0 => Job::Doctor,
            1 => Job::Programmer,
            2 => Job::Clerk,
            3 => Job::PoliceOfficer,
            4 => Job::FireFighter,
            5 => Job::PublicServant,
            6 => Job::Chef,
            7 => Job::Teacher,
            8 => Job::Student,
            _ => unreachable!(),
        }
    }
}

impl Person {
    pub fn generate(
        rng: &mut impl Rng,
        home: Position,
        job_type: Job,
        job_location: Option<Position>,
    ) -> Self {
        Person {
            alive: true,
            sick: false,
            age: rng.gen_range(0..100),
            sex: rng.gen_bool(0.5),
            job_type: Job::generate(rng),
            job_location,
            position: home.clone(),
            home,
            habits: PersonHabits {
                mask: rng.gen_range(0.0..1.0),
                hygiene: rng.gen_range(0.0..1.0),
                acquaintances: HashSet::new(),
            },
        }
    }

    pub fn add_acquaintance(&mut self, id: PersonId) {
        self.habits.acquaintances.insert(id);
    }

    pub fn render(&self, ctx: &mut BTerm) {
        ctx.print_color(self.position.x, self.position.y, LIGHT_BLUE, BLACK, "&");
    }

    pub fn update_action(
        &self,
        world: &World,
        path_cache: &mut PathCache,
        action: &mut PersonAction,
    ) {
        match action {
            PersonAction::AtHome => {
                let work_hours = self.job_type.work_hours();

                if let Some(job_location) = &self.job_location {
                    if world.hours >= work_hours.start && world.hours <= work_hours.end {
                        let path = path_cache.get_path(
                            &world.map,
                            self.home.clone(),
                            job_location.clone(),
                        );

                        *action =
                            PersonAction::Walking(path.clone(), Box::new(PersonAction::Working));
                    }
                }
            }
            PersonAction::Walking(path, next) => {
                if path.len() == 0 {
                    *action = (**next).clone();
                }
            }
            PersonAction::Working => {
                let work_hours = self.job_type.work_hours();

                if world.hours >= work_hours.end {
                    let path =
                        path_cache.get_path(&world.map, self.home.clone(), self.home.clone());

                    *action = PersonAction::Walking(path.clone(), Box::new(PersonAction::AtHome));
                }
            }
        }
    }

    pub fn update(&mut self, id: PersonId, action: &mut PersonAction) -> Option<PersonUpdate> {
        match action {
            PersonAction::AtHome => None,
            PersonAction::Walking(path, _) => {
                self.position = path.pop().unwrap();

                Some(PersonUpdate::Position(id, self.position.clone()))
            }
            PersonAction::Working => None,
        }
    }
}
