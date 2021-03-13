use crate::map::{Map, Position};
use crate::map_generation::MapGenerationSettings;
use crate::person::{Person, PersonAction, PersonId, PersonUpdate};
use crate::world::World;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::sleep;

pub struct PathCache {
    paths: HashMap<(Position, Position), Vec<Position>>,
}

impl PathCache {
    pub fn new() -> Self {
        Self {
            paths: HashMap::new(),
        }
    }

    /// Finds a path between points and chaches the result in `paths`.
    pub fn cache_path(&mut self, map: &Map, start: Position, end: Position) {
        let (path, _cost) = pathfinding::prelude::astar(
            &end,
            |p| {
                let mut neighbors = Vec::new();
                let up = Position::new(p.x, p.y.saturating_sub(1));
                let down = Position::new(p.x, p.y + 1);
                let right = Position::new(p.x + 1, p.y);
                let left = Position::new(p.x.saturating_sub(1), p.y);

                if map.can_walk(&up) {
                    neighbors.push((up, 1));
                }

                if map.can_walk(&down) {
                    neighbors.push((down, 1));
                }

                if map.can_walk(&right) {
                    neighbors.push((right, 1));
                }

                if map.can_walk(&left) {
                    neighbors.push((left, 1));
                }

                neighbors
            },
            |_| 1,
            |p| *p == start,
        )
        .unwrap();

        // hehe xD, shiz fucked, but works better than the alternative
        // double reversed order
        self.paths.insert((start, end), path);
    }

    pub fn get_path(&mut self, map: &Map, start: Position, end: Position) -> &Vec<Position> {
        let key = (start, end);

        if !self.paths.contains_key(&key) {
            self.cache_path(map, key.0.clone(), key.1.clone());
        }

        self.paths.get(&key).unwrap()
    }
}

pub struct GameSession {
    pub player1: PlayerSession,
    pub player2: PlayerSession,
    pub tick_count: u64,
    pub tick_rate: u8,
    pub age: u64,
    pub world: World,
    pub people_actions: HashMap<PersonId, PersonAction>,
    pub path_cache: PathCache,
}

impl GameSession {
    pub async fn send_playload(
        &mut self,
        updates: Vec<WorldUpdate>,
    ) -> Result<(), Box<dyn std::error::Error + 'static + Send + Sync>> {
        // Create payload
        let network_payload = NetworkPayload::create(&self, updates);
        let serialized_payload = bincode::serialize(&network_payload).unwrap();
        let payload_size = (serialized_payload.len() as u32).to_be_bytes();

        // TODO: Consider not awaiting
        self.player1.socket.write_all(&payload_size).await?;
        self.player1.socket.write_all(&serialized_payload).await?;
        self.player2.socket.write_all(&payload_size).await?;
        self.player2.socket.write_all(&serialized_payload).await?;

        Ok(())
    }

    pub fn update(&mut self) -> Vec<WorldUpdate> {
        self.world.set_time(self.age);

        let mut updates = Vec::new();

        for (id, person) in &self.world.people {
            let action = self.people_actions.get_mut(id).unwrap();

            person.update_action(&self.world, &mut self.path_cache, action);
        }

        for (id, person) in &mut self.world.people {
            let action = self.people_actions.get_mut(id).unwrap();

            match person.update(id.clone(), action) {
                Some(u) => updates.push(WorldUpdate::PersonUpdate(u)),
                None => {}
            }
        }

        updates
    }
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
pub enum WorldUpdate {
    SetWorld(World),
    PersonUpdate(PersonUpdate),
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
    pub updates: Vec<WorldUpdate>,
}

impl NetworkPayload {
    pub fn create(state: &GameSession, updates: Vec<WorldUpdate>) -> Self {
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
    let setting = MapGenerationSettings {
        width: 24,
        height: 16,
        structures: crate::structures::STRUCTURES,
    };

    let world = World::generate(setting, &mut rand::thread_rng());//rngs::StdRng::from_seed([132; 32]));

    let people_actions = world
        .people
        .keys()
        .map(|id| (id.clone(), PersonAction::AtHome))
        .collect();

    // Game logic
    let mut state = GameSession {
        player1,
        player2,
        tick_count: 600,
        tick_rate: 20,
        age: 0,
        world,
        people_actions,
        path_cache: PathCache::new(),
    };

    println!("{}", state.player1.socket.peer_addr().unwrap());

    state
        .send_playload(vec![WorldUpdate::SetWorld(state.world.clone())])
        .await?;

    loop {
        // Wait a tick before executing the next loop
        sleep(Duration::from_millis(1000 / state.tick_rate as u64)).await;
        // Count a tick
        state.tick_count = state.tick_count + 1;
        state.age = state.tick_count / state.tick_rate as u64;

        println!("tick");

        let updates = state.update();
        state.send_playload(updates).await?;
    }

    //Ok((player1, player2))
}

#[tokio::main]
pub async fn run() -> Result<(), Box<dyn std::error::Error + 'static + Send + Sync>> {
    // Bind server to host and port
    let listener = TcpListener::bind("0.0.0.0:35565").await?;

    // Init reusable rng
    let mut rng = rand::thread_rng();

    let mut games = Vec::new();

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
        let game_future = tokio::spawn(server_run_game(player1, player2));

        games.push(game_future);
    }

    /*
    for game in games {
        game.await?;
    }
    Ok(())
    */
}
