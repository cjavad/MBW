use crate::map;
use crate::map_generation::MapGenerationSettings;
use crate::person;
use bracket_lib::prelude::*;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Location {
    Home,
    Job(person::Job),
}

impl Location {
    pub fn generate(rng: &mut impl Rng) -> Self {
        match rng.gen_range(0..2) {
            0 => Location::Home,
            1 => Location::Job(person::Job::generate(rng)),
            _ => unreachable!(),
        }
    }
}

/// Incapsulates the entire simulated world.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct World {
    pub map: map::Map,
    pub locations: HashMap<map::Position, Location>,
    pub people: HashMap<person::PersonId, person::Person>,
}

impl World {
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
                map::Tile::Door => Some((map::Position::new(x, y), Location::generate(rng))),
                _ => None,
            })
            .collect::<HashMap<_, _>>();

        // find all homes and collect references
        let mut homes = locations
            .iter()
            .filter_map(|(position, location)| match location {
                Location::Home => Some((position.clone(), Vec::new())),
                _ => None,
            })
            .collect::<HashMap<_, _>>();

        // generate people with random homes
        let mut people: HashMap<person::PersonId, person::Person> = (0..200)
            .into_iter()
            .map(|id| {
                let home = homes.keys().choose(rng).unwrap().clone();
                let id = person::PersonId(id);

                homes.get_mut(&home).unwrap().push(id.clone());

                (id, person::Person::generate(rng, home))
            })
            .collect();

        for (id, person) in &mut people {
            for aq in &homes[&person.home] {
                if id != aq {
                    person.add_acquaintance(aq.clone());
                }
            }
        }

        Self {
            map,
            locations,
            people,
        }
    }

    pub fn render(&self, ctx: &mut BTerm, offset: Point) {
        self.map.render(ctx, offset);

        for (_id, person) in &self.people {
            person.render(ctx);
        }
    }
}
