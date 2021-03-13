mod client;
mod map;
mod map_generation;
mod names;
mod person;
mod server;
mod state;
mod structures;
mod ui;
mod world;

use clap::Clap;

const MAP_WIDTH_CHUNKS: usize = 24;
const MAP_HEIGHT_CHUNKS: usize = 16;

#[derive(Clap)]
#[clap(version = clap::crate_version!(), author = "The Boys")]
pub struct Settings {
    #[clap(short, long)]
    server: bool,
    #[clap(short, long, default_value = "mbwgame.ddns.net:35565")]
    ip: String,
}

fn main() -> Result<(), Box<dyn std::error::Error + 'static + Send + Sync>> {
    let settings = Settings::parse();

    if settings.server {
        server::run(settings.ip)?;
    } else {
        client::run(settings.ip)?;
    }

    Ok(())
}
