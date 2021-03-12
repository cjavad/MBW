use crate::map::Position;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::ops::Range;

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PersonId(u32);

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PersonHabits {
    pub mask: f32,
    pub hygiene: f32,
    pub acquaintances: HashSet<PersonId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Person {
    pub age: u8,
    pub sex: bool,
    pub job: Job,
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
    pub fn generate(rng: &mut impl Rng, homes: &Vec<Position>) -> Self {
        Person {
            age: rng.gen_range(0..100),
            sex: rng.gen_bool(0.5),
            job: Job::generate(rng),
            home: homes.choose(rng).unwrap().clone(),
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
}
