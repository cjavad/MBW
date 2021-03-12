mod client;
mod map;
mod server;
mod state;

fn main() {
    #[cfg(feature = "client")]
    client::run().unwrap();

    #[cfg(feature = "server")]
    server::run();
}