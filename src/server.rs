use crate::person::PersonUpdate;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::sleep;

pub struct GameSession {
    pub player1: PlayerSession,
    pub player2: PlayerSession,
    pub tick_count: u64,
    pub tick_rate: u8,
    pub age: u64,
}

pub struct PlayerSession {
    socket: TcpStream,
    side: bool,
    created: bool,
}

impl PlayerSession {
    pub fn create_player(socket: TcpStream, side: bool) -> Self {
        PlayerSession {
            socket,
            side,
            created: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkPayload {
    /// Unix time in seconds
    pub timestamp: u64,
    /// Current game tick
    pub tick_count: u64,
    /// Age in seconds
    pub age: u64,
    /// Server tickrate
    pub tick_rate: u8,
    /// Vector for PersonUpdate(s)
    pub updates: Vec<PersonUpdate>,
}

impl NetworkPayload {
    pub fn create(state: &GameSession, updates: Vec<PersonUpdate>) -> Self {
        NetworkPayload {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tick_count: state.tick_count,
            age: state.age,
            tick_rate: state.tick_rate,
            updates: updates,
        }
    }
}

async fn server_run_game(
    player1: PlayerSession,
    player2: PlayerSession,
) -> Result<(PlayerSession, PlayerSession), Box<dyn std::error::Error + 'static + Send + Sync>> {
    // Game logic
    let mut state = GameSession {
        player1,
        player2,
        tick_count: 0,
        tick_rate: 20,
        age: 0
    };

    loop {
        println!("{}", state.player1.socket.peer_addr().unwrap());
        // Wait a tick before executing the next loop
        sleep(Duration::from_millis(1000 / state.tick_rate as u64)).await;
        // Count a tick
        state.tick_count = state.tick_count + 1;
        state.age = state.tick_count / state.tick_rate as u64;
        let network_payload = NetworkPayload::create(&state, vec![]);
        let serialized_payload = bincode::serialize(&network_payload).unwrap();
        let payload_size = (serialized_payload.len() as u32).to_be_bytes();
        state.player1.socket.write_all(&payload_size).await?;
        state.player1.socket.write_all(&serialized_payload).await?;
        state.player2.socket.write_all(&payload_size).await?;
        state.player2.socket.write_all(&serialized_payload).await?;
    }

    //Ok((player1, player2))
}

#[tokio::main]
pub async fn run() -> Result<(), Box<dyn std::error::Error + 'static + Send + Sync>> {
    // Bind server to host and port
    let listener = TcpListener::bind("0.0.0.0:35565").await?;

    // Init reusable rng
    let mut rng = rand::thread_rng();

    // Infinite socket loop, at least until two players have connected.
    loop {
        // Wait until a client tries to connect
        let (player1_socket, _) = listener.accept().await?;
        let (player2_socket, _) = listener.accept().await?;

        // Randomly decide sides
        let side = rng.gen_bool(0.5);

        // Init players
        let player1 = PlayerSession::create_player(player1_socket, side);
        let player2 = PlayerSession::create_player(player2_socket, !side);

        // Start game
        tokio::spawn(server_run_game(player1, player2));
    }
}
