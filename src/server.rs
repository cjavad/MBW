use crate::map::{Map, Position, Tile};
use crate::map_generation::MapGenerationSettings;
use crate::person::{Person, PersonAction, PersonId, PersonUpdate};
use crate::world::World;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::io::Interest;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{
    tcp::{OwnedReadHalf, OwnedWriteHalf},
    TcpListener, TcpStream,
};
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

    /// Clears the cache
    pub fn invalidate(&mut self) {
        self.paths.clear();
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
    pub receiver: Receiver<PlayerUpdate>,
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

        self.player1.socket.write_all(&payload_size).await?;
        self.player1.socket.write_all(&serialized_payload).await?;
        self.player2.socket.write_all(&payload_size).await?;
        self.player2.socket.write_all(&serialized_payload).await?;

        Ok(())
    }

    pub async fn update(&mut self, rng: &mut impl Rng) -> Vec<WorldUpdate> {
        self.world.set_time(self.age);

        self.handle_players().await;

        let mut updates = Vec::new();
        let mut person_locations: HashMap<Position, Vec<PersonId>> = HashMap::new();

        for (id, person) in &self.world.people {
            let action = self.people_actions.get_mut(id).unwrap();
            person_locations
                .entry(person.position.clone())
                .or_insert(Vec::new())
                .push(id.clone());
            person.update_action(&self.world, &mut self.path_cache, action);
        }

        for (position, persons) in &person_locations {
            let tile = self.world.map.get_tile(position);

            for other_id in persons {
                let other_person = self.world.people.get(other_id).unwrap().clone();

                for id in persons {
                    let mut infection_chance: f32 = 0.5;

                    // If it's the same person as the inital loop skip
                    if other_id == id {
                        continue;
                    }

                    let person = self.world.people.get_mut(id).unwrap();

                    if person.tick_last_touched - self.tick_count > 20 {
                        person.tick_last_touched = self.tick_count;
                    } else {
                        continue;
                    }

                    // False sex have better immune systems than true sex
                    if !person.sex {
                        infection_chance -= 0.1;
                    }

                    // Check if you and the other people are wearing masks
                    infection_chance /= 2.0;

                    // Check if you and the other people are wearing masks
                    if person.habits.mask > rng.gen_range(0.0..1.0) {
                        if other_person.habits.mask > rng.gen_range(0.0..1.0) {
                            infection_chance /= 2.0;
                        } else {
                            infection_chance /= 10.0;
                        }
                    }

                    // The older you are the worse your immune system is
                    infection_chance += (person.age as u32 / 500) as f32;

                    if person.vaccinated {
                        infection_chance *= 0.05;
                    }

                    if other_person.infected && infection_chance > rng.gen_range(0.0..100.0) {
                        person.infected = true;
                        updates.push(WorldUpdate::PersonUpdate(PersonUpdate::Infected(
                            id.clone(),
                            person.infected,
                        )));
                    }

                    // See if they become friends
                    if person.habits.socialscore > rng.gen_range(0.0..1.0)
                        && !person.habits.acquaintances.contains(other_id)
                    {
                        person.habits.acquaintances.insert(other_id.clone());
                    }
                }
            }
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

    pub async fn handle_players(&mut self) {
        self.receiver.try_iter().for_each(|update| {
            println!("{:?}", update);
        });
    }
}

pub struct PlayerSession {
    socket: OwnedWriteHalf,
    side: bool,
    created: bool,
}

impl PlayerSession {
    pub fn create_player(socket: OwnedWriteHalf, side: bool) -> Self {
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
    pub fn create(session: &GameSession, updates: Vec<WorldUpdate>) -> Self {
        NetworkPayload {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            tick_count: session.tick_count,
            age: session.age,
            tick_rate: session.tick_rate,
            updates: updates,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlayerCommand {
    PartyImpulse(Position),
    AntivaxCampaign(Position),
    Roadblock(Position),
    SocialImpulse(Position),
    EconomicCrash,
    Testcenter(Position),
    Lockdown,
    Vaccinecenter(Position),
    MaskCampaign(Position)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerUpdate {
    side: bool,
    command: PlayerCommand,
}


impl PlayerUpdate {
    pub fn is_valid(&self) -> bool {
        match &self.command {
            PlayerCommand::PartyImpulse(Position) => self.side == true,
            PlayerCommand::AntivaxCampaign(Position) => self.side == true,
            PlayerCommand::Roadblock(Position) => self.side == true,
            PlayerCommand::SocialImpulse(Position) => self.side == true,
            PlayerCommand::EconomicCrash => self.side == true,
            PlayerCommand::Testcenter(Position) => self.side == false,
            PlayerCommand::Lockdown => self.side == false,
            PlayerCommand::Vaccinecenter(Position) => self.side == false,
            PlayerCommand::MaskCampaign(Position) => self.side == false,
            _ => false
        }
    }

    pub fn tile_lookup(&self) -> Option<Tile> {
        match &self.command {
            PlayerCommand::PartyImpulse(Position) => Some(Tile::Empty),
            PlayerCommand::AntivaxCampaign(Position) => Some(Tile::Empty),
            PlayerCommand::Roadblock(Position) => Some(Tile::Empty),
            PlayerCommand::SocialImpulse(Position) => Some(Tile::Empty),
            PlayerCommand::EconomicCrash => None,
            PlayerCommand::Testcenter(Position) => Some(Tile::Empty),
            PlayerCommand::Lockdown => None,
            PlayerCommand::Vaccinecenter(Position) => Some(Tile::Empty),
            PlayerCommand::MaskCampaign(Position) => Some(Tile::Empty),
            _ => None
        }
    }
}

async fn server_listener(
    sender: Sender<PlayerUpdate>,
    mut read: OwnedReadHalf,
    side: bool
) -> Result<(), Box<dyn std::error::Error + 'static + Send + Sync>> {
    loop {
        let mut header = [0; 4];
        read.read_exact(&mut header).await?;
        let mut data = vec![0; u32::from_be_bytes(header) as usize];
        read.read_exact(&mut data).await?;
        let command: PlayerCommand = bincode::deserialize(&data).unwrap();

        sender.send(PlayerUpdate { side, command }).unwrap();
    }
}

async fn server_run_game(
    player1: TcpStream,
    player2: TcpStream,
) -> Result<(), Box<dyn std::error::Error + 'static + Send + Sync>> {
    let setting = MapGenerationSettings {
        width: 24,
        height: 16,
        structures: crate::structures::STRUCTURES,
    };

    let world = World::generate(setting, &mut rand::thread_rng()); //rngs::StdRng::from_seed([132; 32]));

    let people_actions = world
        .people
        .keys()
        .map(|id| (id.clone(), PersonAction::AtHome))
        .collect();

    // Init reusable rng
    let mut rng = rand::rngs::StdRng::from_seed([rand::thread_rng().gen_range(0..=255); 32]);

    // Randomly decide sides
    let side = rng.gen_bool(0.5);

    let (player1_read, player1_write) = player1.into_split();
    let (player2_read, player2_write) = player2.into_split();

    // Init players
    let player1 = PlayerSession::create_player(player1_write, side);
    let player2 = PlayerSession::create_player(player2_write, !side);

    // Game logic
    let (sender, receiver) = channel();
    tokio::spawn(server_listener(sender.clone(), player1_read, player1.side));
    tokio::spawn(server_listener(sender, player2_read, player2.side));

    let mut session = GameSession {
        player1,
        player2,
        tick_count: 600,
        tick_rate: 10,
        age: 0,
        world,
        people_actions,
        path_cache: PathCache::new(),
        receiver,
    };

    session
        .send_playload(vec![WorldUpdate::SetWorld(session.world.clone())])
        .await?;

    loop {
        // Wait a tick before executing the next loop
        sleep(Duration::from_millis(1000 / session.tick_rate as u64)).await;
        // Count a tick
        session.tick_count = session.tick_count + 1;
        session.age = session.tick_count / session.tick_rate as u64;

        let updates = session.update(&mut rng).await;
        session.send_playload(updates).await?;
    }

    //Ok((player1, player2))
}

#[tokio::main]
pub async fn run(ip: String) -> Result<(), Box<dyn std::error::Error + 'static + Send + Sync>> {
    // Bind server to host and port
    let listener = TcpListener::bind(ip).await?;

    let mut games = Vec::new();

    // Infinite socket loop, at least until two players have connected.
    loop {
        // Wait until a client tries to connect
        let (player1_socket, _) = listener.accept().await?;
        let (player2_socket, _) = listener.accept().await?;

        // Start game
        let game_future = tokio::spawn(server_run_game(player1_socket, player2_socket));

        games.push(game_future);
    }

    /*
    for game in games {
        game.await?;
    }
    Ok(())
    */
}
