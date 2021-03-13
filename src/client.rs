use crate::server::NetworkPayload;
use crate::state;
use bracket_lib::prelude::*;
use std::error::Error;
use std::io;
use std::sync::mpsc::{channel, Receiver, Sender};
use tokio::io::AsyncReadExt;
use tokio::io::Interest;
use tokio::net::TcpStream;

pub struct ClientNetworkHandle {
    receiver: Receiver<NetworkPayload>,
}

impl ClientNetworkHandle {
    pub fn get_payloads(&self) -> Vec<NetworkPayload> {
        self.receiver.try_iter().collect()
    }
}

async fn client_main(
    ip: String, 
    sender: Sender<NetworkPayload>,
) -> Result<(), Box<dyn std::error::Error + 'static + Send + Sync>> {
    // Connect to host
    let mut stream = TcpStream::connect(ip).await?;

    loop {
        let ready = stream
            .ready(Interest::READABLE | Interest::WRITABLE)
            .await?;

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
pub async fn run(ip: String) -> Result<(), Box<dyn std::error::Error + 'static + Send + Sync>> {
    // init termial
    let ctx = BTermBuilder::simple(crate::MAP_WIDTH_CHUNKS * 6, crate::MAP_HEIGHT_CHUNKS * 6)?
        .with_title("MBW")
        .with_vsync(true)
        .with_fps_cap(60.0)
        .build()?;

    let (sender, receiver) = channel();
    let handle = ClientNetworkHandle { receiver };

    // Connect to server
    tokio::spawn(client_main(ip, sender));

    // init game state
    let state = state::State::new(handle);

    // run main loop
    main_loop(ctx, state)
}
