use crate::state;
use bracket_lib::prelude::*;

pub fn run() -> Result<(), Box<dyn std::error::Error + 'static + Send + Sync>> {
    let ctx = BTermBuilder::simple(24 * 6, 16 * 6)?
        .with_title("MBW")
        .with_vsync(true)
        .with_fps_cap(60.0)
        .build()?;

    let state = state::State::new();

    main_loop(ctx, state)
}
