use crate::map::{Map, Position, Tile};
use crate::map_generation::MapGenerationSettings;
use crate::person::{self, PersonId, Person};
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

/// Incapsulates the entire simulated world.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    pub minutes: u32,
    pub hours: u8,
    pub days: u8,
    pub map: Map,
    pub locations: HashMap<Position, Location>,
    pub people: HashMap<person::PersonId, person::Person>,
}

impl World {
    pub fn empty(chunks_w: usize, chunks_h: usize) -> Self {
        Self {
            minutes: 0,
            hours: 0,
            days: 0,
            map: Map::fill(chunks_w * 6, chunks_h * 6, Tile::Empty),
            locations: HashMap::new(),
            people: HashMap::new(),
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
                Tile::Door => Some((Position::new(x, y), Location::generate(rng))),
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
                    jobs.entry(job).or_insert(Vec::new()).push(position.clone());
                    None
                }
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

        // add acquaintances based on homes
        for (id, person) in &mut people {
            for aq in &homes[&person.home] {
                if id != aq {
                    person.add_acquaintance(aq.clone());
                }
            }
        }

        Self {
            minutes: 0,
            hours: 0,
            days: 0,
            map,
            locations,
            people,
        }
    }

    pub fn set_time(&mut self, age: u64) {
        const MINUTES_PER_AGE: u32 = 10;

        self.minutes = age as u32 * MINUTES_PER_AGE;

        self.hours = (self.minutes / 60) as u8;
        self.minutes = self.minutes % 60;

        self.days = self.hours / 24;
        self.hours = self.hours % 24;
    }

    pub fn render(&self, ctx: &mut BTerm, person_locations: &HashMap<Position, Vec<PersonId>>, offset: Point) {
        self.map.render(ctx, offset);

        for (location, persons) in person_locations {
            match persons.len() {
                1 => ctx.print_color(location.x, location.y, LIGHT_BLUE, BLACK, "&"),
                _ => {},
            }
        }
    }
}
