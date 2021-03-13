use crate::state;
use tokio::net::TcpStream;
use tokio::io::Interest;
use tokio::io::{AsyncReadExt};
use std::error::Error;
use crate::server::NetworkPayload;
use std::io;
use std::sync::mpsc::{Sender, Receiver, channel};
use bracket_lib::prelude::*;

pub struct ClientNetworkHandle {
    receiver: Receiver<NetworkPayload>
}

impl ClientNetworkHandle {
    pub fn get_payloads(&self) -> Vec<NetworkPayload> {
        self.receiver.iter().collect()
    }
}

async fn client_main(sender: Sender<NetworkPayload>) -> Result<(), Box<dyn std::error::Error + 'static + Send + Sync>> {
    // Connect to host
    let mut stream = TcpStream::connect("127.0.0.1:35565").await?;

    loop {
        let ready = stream.ready(Interest::READABLE | Interest::WRITABLE).await?;

        if ready.is_readable() {
            let mut header = [0; 4];
            stream.read_exact(&mut header).await?;
            let mut data = vec![0; u32::from_be_bytes(header) as usize];
            stream.read_exact(&mut data).await?;
            let payload: NetworkPayload = bincode::deserialize(&data).unwrap();
            sender.send(payload).unwrap();
        }
    }
}

#[tokio::main]
pub async fn run() -> Result<(), Box<dyn std::error::Error + 'static + Send + Sync>> {
    // init termial
    let ctx = BTermBuilder::simple(24 * 6, 16 * 6)?
        .with_title("MBW")
        .with_vsync(true)
        .with_fps_cap(60.0)
        .build()?;

    let (sender, receiver) = channel();
    let handle = ClientNetworkHandle { receiver };
    
    // Connect to server
    tokio::spawn(client_main(sender));

    // init game state
    let state = state::State::new(handle);

    // run main loop
    main_loop(ctx, state)
}
