use crate::person::PersonUpdate;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use bincode;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::time::sleep;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServerState {
    pub tick_count: u64,
    pub tick_rate: u8,
    pub age: u32,
}

pub struct PlayerSession {
    socket: TcpStream,
    side: bool,
    created: bool,
}

impl PlayerSession {
    pub fn create_player1(socket: TcpStream) -> Self {
        PlayerSession {
            socket: socket,
            side: rand::thread_rng().gen_bool(0.5),
            created: true,
        }
    }

    pub fn create_player2(socket: TcpStream, player1: &PlayerSession) -> Self {
        PlayerSession {
            socket: socket,
            side: !player1.side,
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
    /// Age in minutes
    pub age: u32,
    /// Server tickrate
    pub tick_rate: u8,
    /// Vector for PersonUpdate(s)
    pub updates: Vec<PersonUpdate>,
}

impl NetworkPayload {
    pub fn create(state: &ServerState, updates: Vec<PersonUpdate>) -> Self {
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

#[tokio::main]
pub async fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Bind server to host and port
    let listener = TcpListener::bind("0.0.0.0:35565").await?;
    // Inital 2 player variables
    let mut player1: Option<PlayerSession> = None;
    let mut player2: Option<PlayerSession> = None;

    // Infinite socket loop, at least until two players have connected.
    loop {
        // Wait until a client tries to connect
        let (socket, _) = listener.accept().await?;

        // If player 1 is already connected, proceed to connect player two and start the game.
        if player1.is_some() {
            player2 = Some(PlayerSession::create_player2(
                socket,
                &player1.as_ref().unwrap(),
            ));

            // Remove player instances from outside async loop
            let mut player1 = std::mem::replace(&mut player1, None).unwrap();
            let mut player2 = std::mem::replace(&mut player2, None).unwrap();

            tokio::spawn(async move {
                // Game logic
                let mut state = ServerState {
                    tick_count: 0,
                    tick_rate: 20,
                    age: 0,
                };

                loop {
                    println!("{}", player1.socket.peer_addr().unwrap());
                    // Wait a tick before executing the next loop
                    sleep(Duration::from_millis(1000 / state.tick_rate as u64)).await;
                    // Count a tick
                    state.tick_count = state.tick_count + 1;
                    let serialized_state = bincode::serialize(&state).unwrap();
                    player1.socket.write_all(&serialized_state).await;
                    player2.socket.write_all(&serialized_state).await;
                }
            });
        } else {
            // Otherwise connect as player 1
            player1 = Some(PlayerSession::create_player1(socket));
        }
    }
}
