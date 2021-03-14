use crate::map::{Map, Position};
use crate::names::{FIRST_NAMES, LAST_NAMES};
use crate::server::{GameSession, PathCache};
use crate::world::{Time, World};
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
    LifeStatus(PersonId, bool),
    Position(PersonId, Position),
    Infected(PersonId, bool),
    Habits(PersonId, PersonHabits),
    Tested(PersonId, bool),
    Vaccinated(PersonId, bool),
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PersonId(pub u32);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PersonAction {
    Working,
    Walking(Vec<Position>, Box<PersonAction>),
    Shopping(u32),
    AtHome,
    Partying(u32),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonHabits {
    pub mask: f32,
    pub hygiene: f32,
    pub socialscore: f32,
    pub vaccination_bias: f32,
    pub acquaintances: HashSet<PersonId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Person {
    pub alive: bool,
    pub infected: bool,
    pub tick_infected: u64,
    pub tick_last_touched: u64,
    pub tested: bool,
    pub vaccinated: bool,
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
    pub fn work_hours(&self) -> Range<u32> {
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

    pub fn as_str(&self) -> &str {
        match self {
            JobType::Doctor => "Doctor",
            JobType::Programmer => "Programmer",
            JobType::Clerk => "Clerk",
            JobType::PoliceOfficer => "Police Officer",
            JobType::FireFighter => "Fire Fighter",
            JobType::PublicServant => "Public Servant",
            JobType::Chef => "Chef",
            JobType::Teacher => "Teacher",
            JobType::Student => "Student",
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
            infected: false,
            vaccinated: false,
            tested: false,
            tick_infected: 0,
            tick_last_touched: 0,
            first_name: FIRST_NAMES.choose(rng).unwrap().to_string(),
            last_name: LAST_NAMES.choose(rng).unwrap().to_string(),
            age: match rng.gen_range(0.0..1.0) {
                n if n > 0.3 => rng.gen_range(18..40),
                _ => rng.gen_range(40..80),
            },
            sex: rng.gen_bool(0.5),
            job,
            position: home.clone(),
            home,
            habits: PersonHabits {
                mask: rng.gen_range(0.0..1.0),
                hygiene: rng.gen_range(0.0..1.0),
                vaccination_bias: rng.gen_range(0.0..1.0),
                socialscore: rng.gen_range(0.0..0.15),
                acquaintances: HashSet::new(),
            },
        }
    }

    pub fn add_acquaintance(&mut self, id: PersonId) {
        self.habits.acquaintances.insert(id);
    }

    pub fn update_action(
        &self,
        world: &World,
        path_cache: &mut PathCache,
        action: &mut PersonAction,
        rng: &mut impl Rng,
    ) {
        match action {
            PersonAction::AtHome => {
                let work_hours = self.job.ty.work_hours();

                if let Some(job_location) = &self.job.location {
                    // leave when lob starts
                    if world.time.hours >= work_hours.start
                        && world.time.hours < work_hours.end
                        // add some random chance, so everyone doesn't leave at excatly the same time
                        && rng.gen_range(0..10) == 0
                        && !(self.tested && self.infected)
                    {
                        let path = path_cache.get_path(
                            &world.map,
                            self.position.clone(),
                            job_location.clone(),
                        );

                        if let Some(path) = path {
                            *action = PersonAction::Walking(
                                path.clone(),
                                Box::new(PersonAction::Working),
                            );
                        }
                    } else if world.time.hours >= work_hours.end {
                        // randomly go shopping
                        if rng.gen_range(0..1000) == 0 {
                            if let Some(shops) = world.job_locations.get(&JobType::Clerk) {
                                let path = path_cache.get_path(
                                    &world.map,
                                    self.position.clone(),
                                    shops.choose(rng).unwrap().clone(),
                                );

                                if let Some(path) = path {
                                    *action = PersonAction::Walking(
                                        path.clone(),
                                        Box::new(PersonAction::Shopping(
                                            world.time.to_minutes() + rng.gen_range(90..120),
                                        )),
                                    );
                                }
                            }
                        }
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

                // add some random chance, so everyone doesn't leave at excatly the same time
                if world.time.hours >= work_hours.end && rng.gen_range(0..10) == 0 {
                    let path =
                        path_cache.get_path(&world.map, self.position.clone(), self.home.clone());

                    if let Some(path) = path {
                        *action =
                            PersonAction::Walking(path.clone(), Box::new(PersonAction::AtHome));
                    }
                }
            }
            PersonAction::Shopping(time) => {
                if world.time.to_minutes() > *time {
                    let path =
                        path_cache.get_path(&world.map, self.position.clone(), self.home.clone());

                    if let Some(path) = path {
                        *action =
                            PersonAction::Walking(path.clone(), Box::new(PersonAction::AtHome));
                    }
                }
            }
            PersonAction::Partying(time) => {
                let _ = time.saturating_sub(1);
                if *time == 0 {
                    let path =
                        path_cache.get_path(&world.map, self.position.clone(), self.home.clone());

                    if let Some(path) = path {
                        *action =
                            PersonAction::Walking(path.clone(), Box::new(PersonAction::AtHome));
                    }
                }
            }
        }
    }

    pub fn update(&mut self, id: PersonId, action: &mut PersonAction) -> Option<PersonUpdate> {
        match action {
            PersonAction::Walking(path, _) => {
                self.position = path.pop().unwrap();

                Some(PersonUpdate::Position(id, self.position.clone()))
            }
            _ => None,
        }
    }
}
