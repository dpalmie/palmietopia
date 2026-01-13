use wasm_bindgen::prelude::*;
use serde::{Deserialize, Serialize};

#[wasm_bindgen]
pub fn get_welcome_message() -> String {
    "Welcome to Palmietopia!".to_string()
}

// ============ Map Size ============

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum MapSize {
    Tiny,
    Small,
    Medium,
    Large,
    Huge,
}

impl MapSize {
    pub fn radius(&self) -> u32 {
        match self {
            MapSize::Tiny => 2,
            MapSize::Small => 4,
            MapSize::Medium => 6,
            MapSize::Large => 8,
            MapSize::Huge => 10,
        }
    }
}

// ============ Player ============

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum PlayerColor {
    Red,
    Blue,
    Green,
    Yellow,
    Purple,
}

impl PlayerColor {
    pub fn from_index(index: usize) -> Self {
        match index % 5 {
            0 => PlayerColor::Red,
            1 => PlayerColor::Blue,
            2 => PlayerColor::Green,
            3 => PlayerColor::Yellow,
            _ => PlayerColor::Purple,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Player {
    pub id: String,
    pub name: String,
    pub color: PlayerColor,
}

// ============ Lobby ============

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum LobbyStatus {
    Waiting,
    Starting,
    InGame,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Lobby {
    pub id: String,
    pub host_id: String,
    pub players: Vec<Player>,
    pub map_size: MapSize,
    pub max_players: u8,
    pub status: LobbyStatus,
}

impl Lobby {
    pub fn new(id: String, host: Player, map_size: MapSize) -> Self {
        let host_id = host.id.clone();
        Self {
            id,
            host_id,
            players: vec![host],
            map_size,
            max_players: 5,
            status: LobbyStatus::Waiting,
        }
    }

    pub fn can_join(&self) -> bool {
        self.players.len() < self.max_players as usize && self.status == LobbyStatus::Waiting
    }

    pub fn can_start(&self) -> bool {
        self.players.len() >= 2 && self.status == LobbyStatus::Waiting
    }
}

// ============ Game Session ============

pub const DEFAULT_BASE_TIME_MS: u64 = 120_000; // 2 minutes
pub const DEFAULT_INCREMENT_MS: u64 = 45_000;  // 45 seconds

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum GameStatus {
    InProgress,
    Finished,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameSession {
    pub id: String,
    pub map: GameMap,
    pub players: Vec<Player>,
    pub current_turn: usize,
    pub status: GameStatus,
    pub player_times_ms: Vec<u64>,  // Time bank for each player
    pub turn_started_at_ms: u64,
    pub base_time_ms: u64,
    pub increment_ms: u64,
}

impl GameSession {
    pub fn from_lobby(lobby: &Lobby) -> Self {
        let map = GameMap::generate(lobby.map_size.radius());
        let player_count = lobby.players.len();
        Self {
            id: lobby.id.clone(),
            map,
            players: lobby.players.clone(),
            current_turn: 0,
            status: GameStatus::InProgress,
            player_times_ms: vec![DEFAULT_BASE_TIME_MS; player_count], // Each player starts with base time
            turn_started_at_ms: 0, // Set by server when game starts
            base_time_ms: DEFAULT_BASE_TIME_MS,
            increment_ms: DEFAULT_INCREMENT_MS,
        }
    }

    /// End the current player's turn. time_used_ms is how long they took.
    pub fn end_current_turn(&mut self, time_used_ms: u64) {
        let current = self.current_turn;
        // Subtract time used, add increment (chess clock style)
        self.player_times_ms[current] = self.player_times_ms[current]
            .saturating_sub(time_used_ms)
            .saturating_add(self.increment_ms);
        // Advance to next player
        self.current_turn = (current + 1) % self.players.len();
    }

    /// Get the current player's remaining time
    pub fn current_player_time(&self) -> u64 {
        self.player_times_ms[self.current_turn]
    }
}

// ============ Messages ============

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    CreateLobby { player_name: String, map_size: MapSize },
    JoinLobby { lobby_id: String, player_name: String },
    LeaveLobby,
    StartGame,
    ListLobbies,
    EndTurn { game_id: String, player_id: String },
    RejoinGame { game_id: String, player_id: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    LobbyCreated { lobby_id: String, player_id: String },
    JoinedLobby { lobby: Lobby, player_id: String },
    LobbyUpdated { lobby: Lobby },
    LobbyList { lobbies: Vec<Lobby> },
    GameStarted { game: GameSession },
    GameRejoined { game: GameSession },
    PlayerLeft { player_id: String },
    Error { message: String },
    TurnChanged { current_turn: usize, player_times_ms: Vec<u64> },
    TimeTick { player_index: usize, remaining_ms: u64 },
}

/// Terrain types for map tiles
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Terrain {
    Grassland,
    Forest,
    Mountain,
    Water,
    Desert,
}

impl Terrain {
    /// Get a random terrain type
    fn random() -> Self {
        use getrandom::getrandom;
        let mut buf = [0u8; 1];
        getrandom(&mut buf).unwrap();
        match buf[0] % 5 {
            0 => Terrain::Grassland,
            1 => Terrain::Forest,
            2 => Terrain::Mountain,
            3 => Terrain::Water,
            _ => Terrain::Desert,
        }
    }
}

/// A single hex tile with axial coordinates
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Tile {
    pub q: i32,
    pub r: i32,
    pub terrain: Terrain,
}

/// The game map containing all tiles
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GameMap {
    pub tiles: Vec<Tile>,
    pub radius: u32,
}

impl GameMap {
    /// Generate a hexagonal map with the given radius
    pub fn generate(radius: u32) -> Self {
        let mut tiles = Vec::new();
        let r = radius as i32;

        // Generate hexagonal map using axial coordinates
        for q in -r..=r {
            let r1 = (-r).max(-q - r);
            let r2 = r.min(-q + r);
            for r_coord in r1..=r2 {
                tiles.push(Tile {
                    q,
                    r: r_coord,
                    terrain: Terrain::random(),
                });
            }
        }

        GameMap { tiles, radius }
    }
}

/// Generate a tiny map (radius 2, 19 tiles)
#[wasm_bindgen]
pub fn generate_tiny_map() -> String {
    let map = GameMap::generate(2);
    serde_json::to_string(&map).unwrap()
}
