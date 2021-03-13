mod client;
mod map;
mod map_generation;
mod person;
mod server;
mod state;
mod structures;
mod ui;
mod world;

const MAP_WIDTH_CHUNKS: usize = 24;
const MAP_HEIGHT_CHUNKS: usize = 16;

fn main() -> Result<(), Box<dyn std::error::Error + 'static + Send + Sync>> {
    #[cfg(not(feature = "server"))]
    client::run()?;

    #[cfg(feature = "server")]
    server::run()?;

    Ok(())
}
