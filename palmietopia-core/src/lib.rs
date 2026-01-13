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
}

impl GameSession {
    pub fn from_lobby(lobby: &Lobby) -> Self {
        let map = GameMap::generate(lobby.map_size.radius());
        Self {
            id: lobby.id.clone(),
            map,
            players: lobby.players.clone(),
            current_turn: 0,
            status: GameStatus::InProgress,
        }
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    LobbyCreated { lobby_id: String, player_id: String },
    LobbyUpdated { lobby: Lobby },
    LobbyList { lobbies: Vec<Lobby> },
    GameStarted { game: GameSession },
    PlayerLeft { player_id: String },
    Error { message: String },
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
