use crate::map;
use crate::person;
use crate::map_generation::MapGenerationSettings;
use std::collections::HashMap;
use bracket_lib::prelude::*;

/// Incapsulates the entire simulated world.
#[derive(Clone, Debug)]
pub struct World {
    pub map: map::Map,
    pub people: HashMap<person::PersonId, person::Person>,
}

impl World {
    pub fn generate(settings: MapGenerationSettings, rng: &mut impl rand::Rng) -> Self {
        Self {
            map: settings.generate(rng),
            people: todo!(),
        }
    }

    pub fn render(&self, ctx: &mut BTerm, offset: Point) {
        self.map.render(ctx, offset);

        for (_id, person) in &self.people {
            person.render(ctx);
        }
    }
}