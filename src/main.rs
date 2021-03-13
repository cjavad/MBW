mod client;
mod map;
mod map_generation;
mod names;
mod person;
mod server;
mod state;
mod structures;
mod world;

fn main() -> Result<(), Box<dyn std::error::Error + 'static + Send + Sync>> {
    #[cfg(not(feature = "server"))]
    client::run()?;

    #[cfg(feature = "server")]
    server::run()?;

    Ok(())
}
