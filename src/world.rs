use crate::map::{Map, Position, Tile};
use crate::map_generation::MapGenerationSettings;
use crate::person::{self, Person, PersonId};
use bracket_lib::prelude::*;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Location {
    Home,
    Job(person::JobType),
}

impl Location {
    pub fn generate(rng: &mut impl Rng) -> Self {
        match rng.gen_range(0..2) {
            0 => Location::Home,
            1 => Location::Job(person::JobType::generate(rng)),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Time {
    pub minutes: u32,
    pub hours: u32,
    pub days: u8,
}

impl Time {
    pub fn new() -> Self {
        Self {
            minutes: 0,
            hours: 0,
            days: 0,
        }
    }

    pub fn set_minutes(&mut self, minutes: u32) {
        self.minutes = minutes;

        self.hours = (self.minutes / 60) as u32;
        self.minutes = self.minutes % 60;

        self.days = (self.hours / 24) as u8;
        self.hours = self.hours % 24;
    }

    pub fn to_minutes(&self) -> u32 {
        self.minutes + self.hours * 60 + self.days as u32 * 60 * 24
    }
}

/// Incapsulates the entire simulated world.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    pub time: Time,
    pub map: Map,
    pub locations: HashMap<Position, Location>,
    pub people: HashMap<person::PersonId, person::Person>,
    pub job_locations: HashMap<person::JobType, Vec<Position>>,
}

impl World {
    pub fn empty(chunks_w: usize, chunks_h: usize) -> Self {
        Self {
            time: Time::new(),
            map: Map::fill(chunks_w * 6, chunks_h * 6, Tile::Empty),
            locations: HashMap::new(),
            people: HashMap::new(),
            job_locations: HashMap::new(),
        }
    }

    pub fn generate(settings: MapGenerationSettings, rng: &mut impl Rng) -> Self {
        let map = settings.generate(rng);

        // generate locations, from doors
        let locations = map
            .tiles
            .iter()
            .enumerate()
            .map(|(x, c)| c.iter().enumerate().map(move |(y, t)| (x.clone(), y, t)))
            .flatten()
            .filter_map(|(x, y, t)| match t {
                Tile::Door(_) => Some((Position::new(x, y), Location::generate(rng))),
                _ => None,
            })
            .collect::<HashMap<_, _>>();

        let mut jobs = HashMap::new();

        // find all homes and collect references
        let mut homes = locations
            .iter()
            .filter_map(|(position, location)| match location {
                Location::Home => Some((position.clone(), Vec::new())),
                Location::Job(job) => {
                    jobs.entry(job.clone())
                        .or_insert(Vec::new())
                        .push(position.clone());
                    None
                }
                _ => None,
            })
            .collect::<HashMap<_, _>>();

        // generate people with random homes
        let mut people: HashMap<person::PersonId, person::Person> = (0..400)
            .into_iter()
            .map(|id| {
                let home = homes.keys().choose(rng).unwrap().clone();
                let id = person::PersonId(id);

                homes.get_mut(&home).unwrap().push(id.clone());

                let job_type = person::JobType::generate(rng);

                let job_location = jobs
                    .get(&job_type)
                    .map(|locations| locations.choose(rng).unwrap().clone());

                let job = person::Job {
                    ty: job_type,
                    location: job_location,
                };

                (id, person::Person::generate(rng, home, job))
            })
            .collect();

        let ids = people.keys().map(|k| k.clone()).collect::<Vec<_>>();

        for id in &ids {
            for _ in 0..rng.gen_range(2..5) {
                let other_id = ids.choose(rng).unwrap();

                if id != other_id {
                    people
                        .get_mut(id)
                        .unwrap()
                        .add_acquaintance(other_id.clone());
                    people
                        .get_mut(other_id)
                        .unwrap()
                        .add_acquaintance(id.clone());
                }
            }
        }

        for _ in 0..10 {
            people.values_mut().choose(rng).unwrap().infected = true;
        }

        Self {
            time: Time::new(),
            map,
            locations,
            people,
            job_locations: jobs,
        }
    }

    pub fn render(
        &self,
        ctx: &mut BTerm,
        person_locations: &HashMap<Position, Vec<PersonId>>,
        offset: Point,
        side: bool,
    ) {
        self.map.render(ctx, offset);

        for (location, persons) in person_locations {
            let sick = persons
                .iter()
                .map(|p| (self.people[p].infected && (side || self.people[p].tested)) as i32 as f32)
                .sum::<f32>()
                / persons.len() as f32;

            let tested = persons
                .iter()
                .map(|p| self.people[p].tested as i32 as f32)
                .sum::<f32>()
                / persons.len() as f32;

            let vaccinated = persons
                .iter()
                .map(|p| self.people[p].vaccinated as i32 as f32)
                .sum::<f32>()
                / persons.len() as f32;

            let tested_color = match tested {
                n if n == 0.0 => LIGHT_BLUE,
                n if n < 0.5 => DARK_GREEN,
                n if n < 1.0 => GREEN,
                n if n == 1.0 => GREEN2,
                _ => unreachable!(),
            };

            let vaccinated_color = match vaccinated {
                n if n == 0.0 => tested_color,
                n if n < 0.5 => BLUE2,
                n if n < 1.0 => BLUE,
                n if n == 1.0 => DARK_BLUE,
                _ => unreachable!(),
            };

            let color = match sick {
                n if n == 0.0 => vaccinated_color,
                n if n < 0.5 => ORANGE2,
                n if n < 1.0 => ORANGE,
                n if n == 1.0 => DARK_RED,
                _ => unreachable!(),
            };

            let location = Position::new(
                location.x + offset.x as usize,
                location.y + offset.y as usize,
            );

            match persons.len() {
                1 => ctx.print_color(location.x, location.y, color, BLACK, "&"),
                n if n <= 9 => ctx.print_color(location.x, location.y, color, BLACK, n),
                _ => ctx.print_color(location.x, location.y, color, BLACK, "9+"),
            }
        }
    }
}
