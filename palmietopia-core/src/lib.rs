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

// ============ Cities ============

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct City {
    pub id: String,
    pub owner_id: String,
    pub q: i32,
    pub r: i32,
    pub name: String,
}

// ============ Units ============

#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum UnitType {
    Conscript,
}

impl UnitType {
    pub fn base_movement(&self) -> u32 {
        match self {
            UnitType::Conscript => 2,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Unit {
    pub id: String,
    pub owner_id: String,
    pub unit_type: UnitType,
    pub q: i32,
    pub r: i32,
    pub movement_remaining: u32,
}

impl Unit {
    pub fn new(id: String, owner_id: String, unit_type: UnitType, q: i32, r: i32) -> Self {
        Self {
            id,
            owner_id,
            unit_type,
            q,
            r,
            movement_remaining: unit_type.base_movement(),
        }
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
    pub cities: Vec<City>,
    pub units: Vec<Unit>,
    pub current_turn: usize,
    pub status: GameStatus,
    pub player_times_ms: Vec<u64>,
    pub turn_started_at_ms: u64,
    pub base_time_ms: u64,
    pub increment_ms: u64,
}

impl GameSession {
    pub fn from_lobby(lobby: &Lobby) -> Self {
        let map = GameMap::generate(lobby.map_size.radius());
        let player_count = lobby.players.len();
        
        // Generate starting positions for cities
        let starting_positions = Self::calculate_starting_positions(&map, player_count);
        
        // Create cities and units for each player
        let mut cities = Vec::new();
        let mut units = Vec::new();
        
        for (i, player) in lobby.players.iter().enumerate() {
            if let Some((city_q, city_r)) = starting_positions.get(i) {
                // Create city
                cities.push(City {
                    id: format!("city-{}-{}", player.id, i),
                    owner_id: player.id.clone(),
                    q: *city_q,
                    r: *city_r,
                    name: format!("{}'s Capital", player.name),
                });
                
                // Create conscript near city
                if let Some((unit_q, unit_r)) = Self::find_adjacent_land_tile(&map, *city_q, *city_r) {
                    units.push(Unit::new(
                        format!("unit-{}-{}", player.id, 0),
                        player.id.clone(),
                        UnitType::Conscript,
                        unit_q,
                        unit_r,
                    ));
                }
            }
        }
        
        Self {
            id: lobby.id.clone(),
            map,
            players: lobby.players.clone(),
            cities,
            units,
            current_turn: 0,
            status: GameStatus::InProgress,
            player_times_ms: vec![DEFAULT_BASE_TIME_MS; player_count],
            turn_started_at_ms: 0,
            base_time_ms: DEFAULT_BASE_TIME_MS,
            increment_ms: DEFAULT_INCREMENT_MS,
        }
    }

    fn calculate_starting_positions(map: &GameMap, player_count: usize) -> Vec<(i32, i32)> {
        let mut positions = Vec::new();
        let radius = map.radius as f64;
        
        // Get valid tiles (not water or mountain)
        let valid_tiles: Vec<&Tile> = map.tiles.iter()
            .filter(|t| t.terrain != Terrain::Water && t.terrain != Terrain::Mountain)
            .collect();
        
        for i in 0..player_count {
            // Calculate angle for this player's sector
            let angle = (2.0 * std::f64::consts::PI * i as f64) / player_count as f64;
            
            // Target direction vector
            let target_q = (angle.cos() * radius * 0.7) as i32;
            let target_r = (angle.sin() * radius * 0.7) as i32;
            
            // Find valid tile closest to target that's far from existing positions
            let best_tile = valid_tiles.iter()
                .filter(|t| {
                    // Ensure minimum distance from other starting positions
                    positions.iter().all(|(pq, pr)| {
                        Self::hex_distance(t.q, t.r, *pq, *pr) >= (radius as i32 / 2).max(3)
                    })
                })
                .min_by_key(|t| {
                    Self::hex_distance(t.q, t.r, target_q, target_r)
                });
            
            if let Some(tile) = best_tile {
                positions.push((tile.q, tile.r));
            } else if let Some(tile) = valid_tiles.first() {
                positions.push((tile.q, tile.r));
            }
        }
        
        positions
    }

    fn find_adjacent_land_tile(map: &GameMap, q: i32, r: i32) -> Option<(i32, i32)> {
        let neighbors = [
            (q + 1, r), (q - 1, r),
            (q, r + 1), (q, r - 1),
            (q + 1, r - 1), (q - 1, r + 1),
        ];
        
        for (nq, nr) in neighbors {
            if let Some(tile) = map.tiles.iter().find(|t| t.q == nq && t.r == nr) {
                if tile.terrain != Terrain::Water && tile.terrain != Terrain::Mountain {
                    return Some((nq, nr));
                }
            }
        }
        None
    }

    pub fn hex_distance(q1: i32, r1: i32, q2: i32, r2: i32) -> i32 {
        ((q1 - q2).abs() + (r1 - r2).abs() + (q1 + r1 - q2 - r2).abs()) / 2
    }

    pub fn get_terrain_at(&self, q: i32, r: i32) -> Option<Terrain> {
        self.map.tiles.iter().find(|t| t.q == q && t.r == r).map(|t| t.terrain)
    }

    pub fn movement_cost(terrain: Terrain) -> Option<u32> {
        match terrain {
            Terrain::Grassland | Terrain::Forest | Terrain::Desert => Some(1),
            Terrain::Mountain => Some(2),
            Terrain::Water => None, // Impassable
        }
    }

    pub fn can_move_unit(&self, unit_id: &str, to_q: i32, to_r: i32) -> Result<u32, String> {
        let unit = self.units.iter().find(|u| u.id == unit_id)
            .ok_or("Unit not found")?;
        
        // Check if destination tile exists and get terrain
        let terrain = self.get_terrain_at(to_q, to_r)
            .ok_or("Invalid destination")?;
        
        // Check terrain is passable
        let cost = Self::movement_cost(terrain)
            .ok_or("Cannot move to water")?;
        
        // Check distance is exactly 1
        let distance = Self::hex_distance(unit.q, unit.r, to_q, to_r);
        if distance != 1 {
            return Err("Can only move to adjacent tiles".to_string());
        }
        
        // Check unit has enough movement
        if unit.movement_remaining < cost {
            return Err("Not enough movement remaining".to_string());
        }
        
        // Check no other unit occupies the tile
        if self.units.iter().any(|u| u.q == to_q && u.r == to_r) {
            return Err("Tile is occupied".to_string());
        }
        
        Ok(cost)
    }

    pub fn move_unit(&mut self, unit_id: &str, to_q: i32, to_r: i32) -> Result<u32, String> {
        let cost = self.can_move_unit(unit_id, to_q, to_r)?;
        
        let unit = self.units.iter_mut().find(|u| u.id == unit_id)
            .ok_or("Unit not found")?;
        
        unit.q = to_q;
        unit.r = to_r;
        unit.movement_remaining -= cost;
        
        Ok(unit.movement_remaining)
    }

    pub fn reset_movement_for_player(&mut self, player_id: &str) {
        for unit in self.units.iter_mut() {
            if unit.owner_id == player_id {
                unit.movement_remaining = unit.unit_type.base_movement();
            }
        }
    }

    pub fn end_current_turn(&mut self, time_used_ms: u64) {
        let current = self.current_turn;
        self.player_times_ms[current] = self.player_times_ms[current]
            .saturating_sub(time_used_ms)
            .saturating_add(self.increment_ms);
        self.current_turn = (current + 1) % self.players.len();
        
        // Reset movement for the new current player
        let next_player_id = self.players[self.current_turn].id.clone();
        self.reset_movement_for_player(&next_player_id);
    }

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
    MoveUnit { game_id: String, player_id: String, unit_id: String, to_q: i32, to_r: i32 },
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
    TurnChanged { current_turn: usize, player_times_ms: Vec<u64>, units: Vec<Unit> },
    TimeTick { player_index: usize, remaining_ms: u64 },
    UnitMoved { unit_id: String, to_q: i32, to_r: i32, movement_remaining: u32 },
}

/// Terrain types for map tiles
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
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
