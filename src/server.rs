use crate::map::{Map, Position, Tile};
use crate::map_generation::MapGenerationSettings;
use crate::person::{Person, PersonAction, PersonId, PersonUpdate};
use crate::world::World;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
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
    paths: HashMap<(Position, Position), Option<Vec<Position>>>,
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
        let path = pathfinding::prelude::astar(
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
        .map(|(p, _)| p);

        // hehe xD, shiz fucked, but works better than the alternative
        // double reversed order
        self.paths.insert((start, end), path);
    }

    pub fn get_path(
        &mut self,
        map: &Map,
        start: Position,
        end: Position,
    ) -> &Option<Vec<Position>> {
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
    pub test_centers: HashSet<Position>,
    pub vaccine_centers: HashSet<Position>,
}

impl GameSession {
    pub async fn send_playload(
        &mut self,
        updates: Vec<StateUpdate>,
    ) -> Result<(), Box<dyn std::error::Error + 'static + Send + Sync>> {
        // Create payload
        let network_payload = NetworkPayload::create(&self, &self.player1, updates.clone());
        let serialized_payload = bincode::serialize(&network_payload).unwrap();
        let payload_size = (serialized_payload.len() as u32).to_be_bytes();

        self.player1.socket.write_all(&payload_size).await?;
        self.player1.socket.write_all(&serialized_payload).await?;

        let network_payload = NetworkPayload::create(&self, &self.player2, updates);
        let serialized_payload = bincode::serialize(&network_payload).unwrap();
        let payload_size = (serialized_payload.len() as u32).to_be_bytes();

        self.player2.socket.write_all(&payload_size).await?;
        self.player2.socket.write_all(&serialized_payload).await?;

        Ok(())
    }

    pub async fn update(&mut self, rng: &mut impl Rng) -> Vec<StateUpdate> {
        self.world.time.set_minutes(self.tick_count as u32);

        self.player1.money += 1;
        self.player2.money += 1;

        self.handle_players().await;

        let mut updates = Vec::new();
        let mut person_locations: HashMap<Position, Vec<PersonId>> = HashMap::new();

        for (id, person) in &self.world.people {
            let action = self.people_actions.get_mut(id).unwrap();
            person_locations
                .entry(person.position.clone())
                .or_insert(Vec::new())
                .push(id.clone());
            person.update_action(&self.world, &mut self.path_cache, action, rng);
        }

        for test_center in &self.test_centers {
            if let Some(persons) = person_locations.get(test_center) {
                for person in persons {
                    self.world.people.get_mut(person).unwrap().tested = true;
                    updates.push(StateUpdate::PersonUpdate(PersonUpdate::Tested(
                        person.clone(),
                        true,
                    )));
                }
            }
        }

        for vaccine_center in &self.vaccine_centers {
            if let Some(persons) = person_locations.get(vaccine_center) {
                for person in persons {
                    self.world.people.get_mut(person).unwrap().vaccinated = true;
                    updates.push(StateUpdate::PersonUpdate(PersonUpdate::Vaccinated(
                        person.clone(),
                        true,
                    )));
                }
            }
        }

        for (position, persons) in &person_locations {
            let tile = self.world.map.get_tile(position);

            for other_id in persons {
                let other_person = self.world.people.get(other_id).unwrap().clone();

                for id in persons {
                    // If it's the same person as the inital loop skip
                    if other_id == id {
                        continue;
                    }

                    let mut infection_chance: f32 = 0.15;

                    let person = self.world.people.get_mut(id).unwrap();

                    if person.tick_last_touched - self.tick_count > 20 {
                        person.tick_last_touched = self.tick_count;
                    } else {
                        continue;
                    }

                    // False sex have better immune systems than true sex
                    if !person.sex {
                        infection_chance *= 0.9;
                    }

                    if person.infected && person.tested {
                        infection_chance *= 0.05;
                    }

                    // Check if you and the other people are wearing masks
                    if person.habits.mask > rng.gen_range(0.0..1.0) {
                        if other_person.habits.mask > rng.gen_range(0.0..1.0) {
                            infection_chance /= 2.0;
                        } else {
                            infection_chance /= 10.0;
                        }
                    }

                    // The older you are the worse your immune system is
                    infection_chance *= 1.0 + person.age as f32 / 100.0;

                    if person.vaccinated {
                        infection_chance *= 0.05;
                    }

                    if other_person.infected && infection_chance > rng.gen_range(0.0..100.0) {
                        person.infected = true;
                        person.tick_infected = self.tick_count;
                        updates.push(StateUpdate::PersonUpdate(PersonUpdate::Infected(
                            id.clone(),
                            person.infected,
                        )));

                    // If you have been infected for more than a week
                    } else if person.infected && self.tick_count - person.tick_infected > 604800 {
                        // If true, die, otherwise live.
                        if rng.gen_bool((person.age as f64) / 500.0) {
                            person.alive = false;
                            updates.push(StateUpdate::PersonUpdate(PersonUpdate::LifeStatus(
                                id.clone(),
                                person.alive,
                            )));
                        } else {
                            person.infected = false;
                            updates.push(StateUpdate::PersonUpdate(PersonUpdate::Infected(
                                id.clone(),
                                person.infected,
                            )));
                        }
                    }

                    // See if they become friends
                    if person.habits.socialscore > rng.gen_range(0.0..1.0)
                        && !person.habits.acquaintances.contains(other_id)
                    {
                        person.add_acquaintance(other_id.clone());
                        updates.push(StateUpdate::PersonUpdate(PersonUpdate::Habits(
                            id.clone(),
                            person.habits.clone(),
                        )));
                    }
                }
            }
        }

        for (id, person) in &mut self.world.people {
            let action = self.people_actions.get_mut(id).unwrap();

            match person.update(id.clone(), action) {
                Some(u) => updates.push(StateUpdate::PersonUpdate(u)),
                None => {}
            }
        }

        updates
    }

    pub async fn handle_players(&mut self) {
        let mut updates = Vec::new();
        let world = &mut self.world;
        let people_actions = &mut self.people_actions;
        let path_cache = &mut self.path_cache;
        let player1 = &mut self.player1;
        let player2 = &mut self.player2;
        let test_centers = &mut self.test_centers;
        let vaccine_centers = &mut self.vaccine_centers;
        self.receiver.try_iter().for_each(|update| {
            println!("{:?}", update);

            if update.is_valid() {
                let money = match &update.player {
                    Player::Player1 => &mut player1.money,
                    Player::Player2 => &mut player2.money,
                };

                let price = update.command.price_lookup();

                if *money < price {
                    return;
                }

                match &update.command {
                    PlayerCommand::PartyImpulse(id) => {
                        *money -= price;

                        let person = world.people.get(&id).unwrap();
                        let action = people_actions.get_mut(&id).unwrap();
                        let path = path_cache.get_path(
                            &world.map,
                            person.position.clone(),
                            person.home.clone(),
                        );
                        if let Some(path) = path {
                            *action = PersonAction::Walking(
                                path.clone(),
                                Box::new(PersonAction::Partying(300)),
                            );
                        }

                        for id in &person.habits.acquaintances {
                            let acquaintance = world.people.get(&id).unwrap();
                            let action = people_actions.get_mut(&id).unwrap();
                            let path = path_cache.get_path(
                                &world.map,
                                acquaintance.position.clone(),
                                person.home.clone(),
                            );

                            if let Some(path) = path {
                                *action = PersonAction::Walking(
                                    path.clone(),
                                    Box::new(PersonAction::Partying(300)),
                                );
                            }
                        }
                    }
                    PlayerCommand::AntivaxCampaign(position) => {
                        if world.map.tiles[position.x][position.y]
                            == update.command.tile_lookup()[0]
                        {
                            *money -= price;

                            world.map.tiles[position.x][position.y] = Tile::AntivaxCampain(480);
                            updates.push(StateUpdate::TileUpdate(
                                position.clone(),
                                world.map.tiles[position.x][position.y],
                            ));
                            path_cache.invalidate();
                        }
                    }
                    PlayerCommand::Roadblock(position) => {
                        if world.map.tiles[position.x][position.y]
                            == update.command.tile_lookup()[0]
                        {
                            *money -= price;

                            world.map.tiles[position.x][position.y] = Tile::RoadBlock;
                            updates.push(StateUpdate::TileUpdate(
                                position.clone(),
                                world.map.tiles[position.x][position.y],
                            ));
                            path_cache.invalidate();
                        } else if world.map.tiles[position.x][position.y]
                            == update.command.tile_lookup()[1]
                        {
                            *money -= price;

                            world.map.tiles[position.x][position.y] = Tile::Empty;
                            updates.push(StateUpdate::TileUpdate(
                                position.clone(),
                                world.map.tiles[position.x][position.y],
                            ));
                            path_cache.invalidate();
                        }
                    }
                    PlayerCommand::SocialImpulse(position) => {
                        if world.map.tiles[position.x][position.y]
                            == update.command.tile_lookup()[0]
                        {
                            *money -= price;

                            updates.push(StateUpdate::TileUpdate(
                                position.clone(),
                                world.map.tiles[position.x][position.y],
                            ));
                            path_cache.invalidate();
                        }
                    }
                    PlayerCommand::EconomicCrash => {
                        *money -= price;

                        test_centers.clear();

                        for x in 0..world.map.width {
                            for y in 0..world.map.height {
                                let tile = &mut world.map.tiles[x][y];
                                match tile {
                                    Tile::MaskCampain(_) => {
                                        *tile = Tile::Empty;
                                        updates.push(StateUpdate::TileUpdate(
                                            Position { x, y },
                                            tile.clone(),
                                        ));
                                    }
                                    Tile::TestCenter => {
                                        *tile = Tile::Empty;
                                        updates.push(StateUpdate::TileUpdate(
                                            Position { x, y },
                                            tile.clone(),
                                        ));
                                    }
                                    Tile::AntivaxCampain(_) => {
                                        *tile = Tile::Empty;
                                        updates.push(StateUpdate::TileUpdate(
                                            Position { x, y },
                                            tile.clone(),
                                        ));
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    PlayerCommand::Testcenter(position) => {
                        if world.map.tiles[position.x][position.y]
                            == update.command.tile_lookup()[0]
                        {
                            *money -= price;

                            world.map.tiles[position.x][position.y] = Tile::TestCenter;
                            updates.push(StateUpdate::TileUpdate(
                                position.clone(),
                                world.map.tiles[position.x][position.y],
                            ));
                            test_centers.insert(position.clone());
                        }
                    }
                    PlayerCommand::Lockdown(position) => {
                        if world.map.tiles[position.x][position.y]
                            == update.command.tile_lookup()[0]
                        {
                            *money -= price;
                            let mut people_in_lockdown: HashSet<PersonId> = HashSet::new();

                            for (id, person) in &world.people {
                                let person = world.people.get(&id).unwrap();
                                if *position == person.home {
                                    people_in_lockdown.insert(id.clone());
                                }
                            }

                            for id in people_in_lockdown {
                                let person = world.people.get_mut(&id).unwrap();
                                person.lockdown = true;

                                let action = people_actions.get_mut(&id).unwrap();
                                let path = path_cache.get_path(
                                    &world.map,
                                    person.position.clone(),
                                    person.home.clone(),
                                );

                                if let Some(path) = path {
                                    *action = PersonAction::Walking(
                                        path.clone(),
                                        Box::new(PersonAction::Lockdown(1440)),
                                    );
                                }
                            }

                            world.map.tiles[position.x][position.y] = Tile::Door(Some(1440));
                            updates.push(StateUpdate::TileUpdate(
                                position.clone(),
                                world.map.tiles[position.x][position.y],
                            ));
                            path_cache.invalidate();
                        }
                    }
                    PlayerCommand::Vaccinecenter(position) => {
                        if world.map.tiles[position.x][position.y]
                            == update.command.tile_lookup()[0]
                        {
                            *money -= price;

                            world.map.tiles[position.x][position.y] = Tile::VaccineCenter;
                            updates.push(StateUpdate::TileUpdate(
                                position.clone(),
                                world.map.tiles[position.x][position.y],
                            ));

                            vaccine_centers.insert(position.clone());
                        }
                    }
                    PlayerCommand::MaskCampaign(position) => {
                        if world.map.tiles[position.x][position.y]
                            == update.command.tile_lookup()[0]
                        {
                            *money -= price;

                            world.map.tiles[position.x][position.y] = Tile::MaskCampain(480);
                            updates.push(StateUpdate::TileUpdate(
                                position.clone(),
                                world.map.tiles[position.x][position.y],
                            ));
                            path_cache.invalidate();
                        }
                    }
                }
            }
        });
        self.send_playload(updates).await.unwrap();
    }

    pub fn is_game_over_and_who_won(&self) -> Option<bool> {
        let mut all_total = 0;
        let mut all_infected = 0;

        for (_id, person) in &self.world.people {
            all_total += 1;
            if person.infected {
                all_infected += 1;
            }
        }

        if self.world.time.days > 3 {
            if all_total as f32 / all_infected as f32 >= 2.0 {
                Some(true)
            } else {
                Some(false)
            }
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Player {
    Player1,
    Player2,
}

pub struct PlayerSession {
    socket: OwnedWriteHalf,
    side: bool,
    created: bool,
    money: u32,
}

impl PlayerSession {
    pub fn create_player(socket: OwnedWriteHalf, side: bool) -> Self {
        PlayerSession {
            socket,
            side,
            created: true,
            money: 0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum StateUpdate {
    SetWorld(World),
    TileUpdate(Position, Tile),
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
    /// The side the player is on
    pub side: bool,
    /// The amount of the players owns
    pub money: u32,
    /// Vector for PersonUpdate(s)
    pub updates: Vec<StateUpdate>,
}

impl NetworkPayload {
    pub fn create(
        session: &GameSession,
        player_session: &PlayerSession,
        updates: Vec<StateUpdate>,
    ) -> Self {
        NetworkPayload {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            side: player_session.side,
            money: player_session.money,
            tick_count: session.tick_count,
            age: session.age,
            tick_rate: session.tick_rate,
            updates: updates,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PlayerCommand {
    PartyImpulse(PersonId), // Placing a party impulse makes all the people connected to it (employes and acquaintances) come together
    AntivaxCampaign(Position), // Sets Person.vaccine to -1 (no more vaccines) and lowers Person.habits.mask to 0
    Roadblock(Position), // Disable routes to test centers or any other place for that matter (at position)
    SocialImpulse(Position), // Creates hotspot at Position people will flock about, increases habits.acquaintances
    EconomicCrash, // Disables test centers and mask campaigns (also antivaccampaigns and partyimpulse)
    Testcenter(Position), // Puts household and acquaintances in lockdown if postive, increases Person.habits.hygiene
    Lockdown(Position),   // People in door (building) are stuck
    Vaccinecenter(Position), // Sets Person.vaccine to 1 when person passes position
    MaskCampaign(Position), // Sets Person.habits.mask to 1 when person passses positions
}

impl PlayerCommand {
    pub fn is_valid(&self, side: bool) -> bool {
        match self {
            PlayerCommand::PartyImpulse(_) => side == true,
            PlayerCommand::AntivaxCampaign(_) => side == true,
            PlayerCommand::Roadblock(_) => true,
            PlayerCommand::SocialImpulse(_) => side == true,
            PlayerCommand::EconomicCrash => side == true,
            PlayerCommand::Testcenter(_) => side == false,
            PlayerCommand::Lockdown(_) => side == false,
            PlayerCommand::Vaccinecenter(_) => side == false,
            PlayerCommand::MaskCampaign(_) => side == false,
            _ => false,
        }
    }

    pub fn tile_lookup(&self) -> &[Tile] {
        match self {
            PlayerCommand::PartyImpulse(_) => &[Tile::Empty],
            PlayerCommand::AntivaxCampaign(_) => &[Tile::Empty],
            PlayerCommand::Roadblock(_) => &[Tile::Empty, Tile::RoadBlock],
            PlayerCommand::SocialImpulse(_) => &[Tile::Empty],
            PlayerCommand::EconomicCrash => &[],
            PlayerCommand::Testcenter(_) => &[Tile::Empty],
            PlayerCommand::Lockdown(_) => &[Tile::Door(None)],
            PlayerCommand::Vaccinecenter(_) => &[Tile::Empty],
            PlayerCommand::MaskCampaign(_) => &[Tile::Empty],
        }
    }

    pub fn price_lookup(&self) -> u32 {
        match self {
            PlayerCommand::PartyImpulse(_) => 250,
            PlayerCommand::AntivaxCampaign(_) => 800,
            PlayerCommand::Roadblock(_) => 80,
            PlayerCommand::SocialImpulse(_) => 180,
            PlayerCommand::EconomicCrash => 800,
            PlayerCommand::Testcenter(_) => 300,
            PlayerCommand::Lockdown(_) => 100,
            PlayerCommand::Vaccinecenter(_) => 600,
            PlayerCommand::MaskCampaign(_) => 200,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlayerUpdate {
    side: bool,
    command: PlayerCommand,
    player: Player,
}

impl PlayerUpdate {
    pub fn is_valid(&self) -> bool {
        self.command.is_valid(self.side)
    }
}

async fn server_listener(
    player: Player,
    sender: Sender<PlayerUpdate>,
    mut read: OwnedReadHalf,
    side: bool,
) -> Result<(), Box<dyn std::error::Error + 'static + Send + Sync>> {
    loop {
        let mut header = [0; 4];
        read.read_exact(&mut header).await?;
        let mut data = vec![0; u32::from_be_bytes(header) as usize];
        read.read_exact(&mut data).await?;
        let command: PlayerCommand = bincode::deserialize(&data).unwrap();

        sender
            .send(PlayerUpdate {
                side,
                command,
                player: player.clone(),
            })
            .unwrap();
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
    tokio::spawn(server_listener(
        Player::Player1,
        sender.clone(),
        player1_read,
        player1.side,
    ));
    tokio::spawn(server_listener(
        Player::Player2,
        sender,
        player2_read,
        player2.side,
    ));

    let mut session = GameSession {
        player1,
        player2,
        tick_count: 120,
        tick_rate: 10,
        age: 0,
        world,
        people_actions,
        path_cache: PathCache::new(),
        receiver,
        test_centers: HashSet::new(),
        vaccine_centers: HashSet::new(),
    };

    session
        .send_playload(vec![StateUpdate::SetWorld(session.world.clone())])
        .await?;

    loop {
        // Wait a tick before executing the next loop
        sleep(Duration::from_millis(1000 / session.tick_rate as u64)).await;
        // Count a tick
        session.tick_count = session.tick_count + 1;
        session.age = session.tick_count / session.tick_rate as u64;

        let updates = session.update(&mut rng).await;
        session.send_playload(updates).await?;

        match session.is_game_over_and_who_won() {
            Some(false) => {
                println!("President won!");
                panic!()
            }
            Some(true) => {
                println!("Virus won!");
                panic!()
            }
            _ => {}
        }
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
