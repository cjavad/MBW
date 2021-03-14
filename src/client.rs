use crate::server::{NetworkPayload, PlayerCommand};
use crate::state;
use bracket_lib::prelude::*;
use std::error::Error;
use std::io;
use std::sync::mpsc::{channel, Receiver, Sender};
use tokio::io::Interest;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct ClientNetworkHandle {
    receiver: Receiver<NetworkPayload>,
}

impl ClientNetworkHandle {
    pub fn get_payloads(&self) -> Vec<NetworkPayload> {
        self.receiver.try_iter().collect()
    }
}

pub struct PlayerCommandHandle {
    sender: Sender<PlayerCommand>,
}

async fn client_main(
    ip: String,
    sender: Sender<NetworkPayload>,
    receiver: Receiver<PlayerCommand>,
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

        if ready.is_writable() {
            let collected: Vec<PlayerCommand> = receiver.try_iter().collect();
            for update in collected {
                let serialized_payload = bincode::serialize(&update).unwrap();
                let payload_size = (serialized_payload.len() as u32).to_be_bytes();
                stream.write_all(&payload_size).await?;
                stream.write_all(&serialized_payload).await?;
            }
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

    // Client network queue for server tick updates
    let (client_sender, client_receiver) = channel();
    let client_handle = ClientNetworkHandle {
        receiver: client_receiver,
    };
    // Player network queue for player action updates
    let (player_sender, player_receiver) = channel();
    let player_handle = PlayerCommandHandle {
        sender: player_sender,
    };

    // Connect to server
    tokio::spawn(client_main(ip, client_sender, player_receiver));

    // init game state
    let state = state::State::new(client_handle);
    player_handle.sender.send(PlayerCommand::Lockdown).unwrap();

    // run main loop
    main_loop(ctx, state)
}
