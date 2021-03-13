use crate::state;
use bracket_lib::prelude::*;

pub fn run() -> Result<(), Box<dyn std::error::Error + 'static + Send + Sync>> {
    // init termial
    let ctx = BTermBuilder::simple(24 * 6, 16 * 6)?
        .with_title("MBW")
        .with_vsync(true)
        .with_fps_cap(60.0)
        .build()?;

    // TODO: connect to server

    // init game state
    let state = state::State::new();

    // run main loop
    main_loop(ctx, state)
}
