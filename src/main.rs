use bracket_lib::prelude::*;

pub struct State {

}

impl State {
    pub fn new() -> Self {
        Self {}
    }
}

impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        ctx.cls();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error + 'static + Send + Sync>> {
    let ctx = BTermBuilder::simple(200, 100)?.with_title("MBW").build()?;

    let state = State::new();

    main_loop(ctx, state)
}