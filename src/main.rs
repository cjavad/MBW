mod client;
mod map;
mod map_generation;
mod person;
mod server;
mod state;
mod structures;

fn main() {
    #[cfg(not(feature = "server"))]
    client::run().unwrap();

    #[cfg(feature = "server")]
    server::run();
}
