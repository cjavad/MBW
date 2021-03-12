mod client;
mod map;
mod server;
mod state;

fn main() {
    #[cfg(client)]
    client::run().unwrap();

    #[cfg(server)]
    server::run();
}