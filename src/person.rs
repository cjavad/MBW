use crate::map::{Map, Position};
use crate::server::{GameSession, PathCache};
use crate::names::{FIRST_NAMES, LAST_NAMES};
use crate::world::World;
use bracket_lib::prelude::*;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ops::Range;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum JobType {
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
pub struct Job {
    pub ty: JobType,
    pub location: Option<Position>,
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
    pub socialscore: f32,
    pub acquaintances: HashSet<PersonId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Person {
    pub alive: bool,
    pub sick: bool,
    pub first_name: String,
    pub last_name: String,
    pub age: u8,
    pub sex: bool,
    pub job: Job,
    pub position: Position,
    pub home: Position,
    pub habits: PersonHabits,
}

impl JobType {
    pub fn work_hours(&self) -> Range<u8> {
        match self {
            JobType::Doctor => 9..17,
            JobType::Programmer => 12..22,
            JobType::Clerk => 7..18,
            JobType::PoliceOfficer => 6..16,
            JobType::FireFighter => 11..21,
            JobType::PublicServant => 11..23,
            JobType::Chef => 14..23,
            JobType::Teacher => 8..17,
            JobType::Student => 8..16,
        }
    }

    pub fn generate(rng: &mut impl Rng) -> Self {
        match rng.gen_range(0..9) {
            0 => JobType::Doctor,
            1 => JobType::Programmer,
            2 => JobType::Clerk,
            3 => JobType::PoliceOfficer,
            4 => JobType::FireFighter,
            5 => JobType::PublicServant,
            6 => JobType::Chef,
            7 => JobType::Teacher,
            8 => JobType::Student,
            _ => unreachable!(),
        }
    }
}

impl Person {
    pub fn generate(rng: &mut impl Rng, home: Position, job: Job) -> Self {
        Person {
            alive: true,
            sick: false,
            first_name: FIRST_NAMES.choose(rng).unwrap().to_string(),
            last_name: LAST_NAMES.choose(rng).unwrap().to_string(),
            age: rng.gen_range(1..100),
            sex: rng.gen_bool(0.5),
            job,
            position: home.clone(),
            home,
            habits: PersonHabits {
                mask: rng.gen_range(0.0..1.0),
                hygiene: rng.gen_range(0.0..1.0),
                socialscore: rng.gen_range(0.0..0.15),
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
                let work_hours = self.job.ty.work_hours();

                if let Some(job_location) = &self.job.location {
                    if world.hours >= work_hours.start && world.hours < work_hours.end {
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
                let work_hours = self.job.ty.work_hours();

                if let Some(job_location) = &self.job.location {
                    if world.hours >= work_hours.end {
                        let path = path_cache.get_path(
                            &world.map,
                            job_location.clone(),
                            self.home.clone(),
                        );

                        *action =
                            PersonAction::Walking(path.clone(), Box::new(PersonAction::AtHome));
                    }
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
