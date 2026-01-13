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
    pub is_capitol: bool,
    pub produced_this_turn: bool,
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

    /// Returns (max_hp, attack, defense)
    pub fn stats(&self) -> (u32, u32, u32) {
        match self {
            UnitType::Conscript => (50, 25, 15),
        }
    }

    pub fn cost(&self) -> u64 {
        match self {
            UnitType::Conscript => 25,
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
    pub hp: u32,
    pub max_hp: u32,
}

impl Unit {
    pub fn new(id: String, owner_id: String, unit_type: UnitType, q: i32, r: i32) -> Self {
        let (max_hp, _, _) = unit_type.stats();
        Self {
            id,
            owner_id,
            unit_type,
            q,
            r,
            movement_remaining: unit_type.base_movement(),
            hp: max_hp,
            max_hp,
        }
    }

    pub fn attack(&self) -> u32 {
        self.unit_type.stats().1
    }

    pub fn defense(&self) -> u32 {
        self.unit_type.stats().2
    }
}

// ============ Game Session ============

pub const DEFAULT_BASE_TIME_MS: u64 = 120_000; // 2 minutes
pub const DEFAULT_INCREMENT_MS: u64 = 45_000;  // 45 seconds
pub const STARTING_GOLD: u64 = 50;
pub const BASE_INCOME: u64 = 20;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum GameStatus {
    InProgress,
    Victory { winner_id: String },
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
    pub eliminated_players: Vec<String>,
    pub player_times_ms: Vec<u64>,
    pub player_gold: Vec<u64>,
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
                // Create capitol city
                cities.push(City {
                    id: format!("city-{}-{}", player.id, i),
                    owner_id: player.id.clone(),
                    q: *city_q,
                    r: *city_r,
                    name: format!("{}'s Capital", player.name),
                    is_capitol: true,
                    produced_this_turn: false,
                });
                
                // Create conscript in the capitol city
                units.push(Unit::new(
                    format!("unit-{}-{}", player.id, 0),
                    player.id.clone(),
                    UnitType::Conscript,
                    *city_q,
                    *city_r,
                ));
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
            eliminated_players: Vec::new(),
            player_times_ms: vec![DEFAULT_BASE_TIME_MS; player_count],
            player_gold: vec![STARTING_GOLD; player_count],
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

    pub fn move_unit(&mut self, unit_id: &str, to_q: i32, to_r: i32) -> Result<MoveOutcome, String> {
        let cost = self.can_move_unit(unit_id, to_q, to_r)?;
        
        let unit = self.units.iter_mut().find(|u| u.id == unit_id)
            .ok_or("Unit not found")?;
        
        let attacker_owner = unit.owner_id.clone();
        unit.q = to_q;
        unit.r = to_r;
        unit.movement_remaining -= cost;
        let movement_remaining = unit.movement_remaining;
        
        // Check for city capture
        let (captured_city, eliminated_player) = self.try_capture_city(to_q, to_r, &attacker_owner);
        
        Ok(MoveOutcome {
            movement_remaining,
            captured_city,
            eliminated_player,
        })
    }
    
    /// Try to capture a city at the given position. Returns (captured_city, eliminated_player).
    fn try_capture_city(&mut self, q: i32, r: i32, new_owner: &str) -> (Option<City>, Option<String>) {
        let city_idx = self.cities.iter().position(|c| c.q == q && c.r == r);
        
        let Some(idx) = city_idx else {
            return (None, None);
        };
        
        let old_owner = self.cities[idx].owner_id.clone();
        
        // Can't capture your own city
        if old_owner == new_owner {
            return (None, None);
        }
        
        let is_capitol = self.cities[idx].is_capitol;
        let mut eliminated_player = None;
        
        if is_capitol {
            // Eliminate the player!
            eliminated_player = Some(old_owner.clone());
            self.eliminated_players.push(old_owner.clone());
            
            // Transfer all their cities
            for c in self.cities.iter_mut() {
                if c.owner_id == old_owner {
                    c.owner_id = new_owner.to_string();
                    if c.is_capitol && !(c.q == q && c.r == r) {
                        c.is_capitol = false;
                    }
                }
            }
            
            // Remove all their units
            self.units.retain(|u| u.owner_id != old_owner);
            
            // Check for victory
            let remaining_players: Vec<_> = self.players.iter()
                .filter(|p| !self.eliminated_players.contains(&p.id))
                .collect();
            if remaining_players.len() == 1 {
                self.status = GameStatus::Victory { winner_id: remaining_players[0].id.clone() };
            }
        } else {
            // Just capture the city
            self.cities[idx].owner_id = new_owner.to_string();
        }
        
        let captured_city = Some(self.cities[idx].clone());
        (captured_city, eliminated_player)
    }

    pub fn reset_movement_for_player(&mut self, player_id: &str) {
        for unit in self.units.iter_mut() {
            if unit.owner_id == player_id {
                unit.movement_remaining = unit.unit_type.base_movement();
            }
        }
    }

    pub fn fortify_unit(&mut self, unit_id: &str) -> Result<u32, String> {
        let unit = self.units.iter_mut().find(|u| u.id == unit_id)
            .ok_or("Unit not found")?;
        
        // Must have full movement (hasn't acted this turn)
        if unit.movement_remaining < unit.unit_type.base_movement() {
            return Err("Cannot fortify after moving".to_string());
        }
        
        // Heal 25% of max HP
        let heal_amount = unit.max_hp / 4;
        unit.hp = (unit.hp + heal_amount).min(unit.max_hp);
        unit.movement_remaining = 0;
        
        Ok(unit.hp)
    }

    pub fn buy_unit(&mut self, player_id: &str, city_id: &str, unit_type: UnitType) -> Result<Unit, String> {
        // Find player index
        let player_idx = self.players.iter().position(|p| p.id == player_id)
            .ok_or("Player not found")?;
        
        // Find city
        let city_idx = self.cities.iter().position(|c| c.id == city_id)
            .ok_or("City not found")?;
        
        // Extract city info first to avoid borrow issues
        let city = &self.cities[city_idx];
        let city_q = city.q;
        let city_r = city.r;
        let city_owner = city.owner_id.clone();
        let city_produced = city.produced_this_turn;
        
        // Validate ownership
        if city_owner != player_id {
            return Err("Not your city".to_string());
        }
        
        // Check if city already produced this turn
        if city_produced {
            return Err("City has already produced this turn".to_string());
        }
        
        // Check if city is unoccupied
        if self.units.iter().any(|u| u.q == city_q && u.r == city_r) {
            return Err("City is occupied by a unit".to_string());
        }
        
        // Check gold
        let cost = unit_type.cost();
        if self.player_gold[player_idx] < cost {
            return Err("Not enough gold".to_string());
        }
        
        // Deduct gold
        self.player_gold[player_idx] -= cost;
        
        // Mark city as produced
        self.cities[city_idx].produced_this_turn = true;
        
        // Create unit with 0 movement - generate random ID
        let mut rand_bytes = [0u8; 8];
        getrandom::getrandom(&mut rand_bytes).unwrap();
        let rand_num = u64::from_le_bytes(rand_bytes);
        let unit_id = format!("unit-{}-{:x}", player_id, rand_num);
        let mut unit = Unit::new(
            unit_id,
            player_id.to_string(),
            unit_type,
            city_q,
            city_r,
        );
        unit.movement_remaining = 0; // Can't move on turn created
        
        self.units.push(unit.clone());
        
        Ok(unit)
    }

    pub fn end_current_turn(&mut self, time_used_ms: u64) {
        let current = self.current_turn;
        self.player_times_ms[current] = self.player_times_ms[current]
            .saturating_sub(time_used_ms)
            .saturating_add(self.increment_ms);
        
        // Grant income to the player who just finished their turn
        self.player_gold[current] += BASE_INCOME;
        
        // Skip eliminated players
        loop {
            self.current_turn = (self.current_turn + 1) % self.players.len();
            let next_player_id = &self.players[self.current_turn].id;
            if !self.eliminated_players.contains(next_player_id) {
                break;
            }
            // Safety: if all players eliminated except one, we'd have victory already
            if self.current_turn == current {
                break;
            }
        }
        
        // Reset movement for the new current player
        let next_player_id = self.players[self.current_turn].id.clone();
        self.reset_movement_for_player(&next_player_id);
        
        // Reset production for new current player's cities
        for city in self.cities.iter_mut() {
            if city.owner_id == next_player_id {
                city.produced_this_turn = false;
            }
        }
    }

    pub fn current_player_time(&self) -> u64 {
        self.player_times_ms[self.current_turn]
    }

    /// Check if a unit is garrisoned (standing on own city)
    pub fn is_unit_garrisoned(&self, unit: &Unit) -> bool {
        self.cities.iter().any(|c| c.q == unit.q && c.r == unit.r && c.owner_id == unit.owner_id)
    }

    /// Get effective defense for a unit (with garrison bonus)
    pub fn effective_defense(&self, unit: &Unit) -> u32 {
        let base = unit.defense();
        if self.is_unit_garrisoned(unit) {
            base + base / 2 // +50% defense when garrisoned
        } else {
            base
        }
    }

    /// Combat result struct
    pub fn resolve_combat(&mut self, attacker_id: &str, defender_id: &str) -> Result<CombatOutcome, String> {
        // Find units
        let attacker_idx = self.units.iter().position(|u| u.id == attacker_id)
            .ok_or("Attacker not found")?;
        let defender_idx = self.units.iter().position(|u| u.id == defender_id)
            .ok_or("Defender not found")?;
        
        // Check they are adjacent
        let attacker = &self.units[attacker_idx];
        let defender = &self.units[defender_idx];
        let distance = Self::hex_distance(attacker.q, attacker.r, defender.q, defender.r);
        if distance != 1 {
            return Err("Units must be adjacent to attack".to_string());
        }
        
        // Check attacker has movement
        if attacker.movement_remaining == 0 {
            return Err("No movement remaining to attack".to_string());
        }
        
        // Calculate damage
        let attacker_attack = self.units[attacker_idx].attack();
        let defender_effective_def = self.effective_defense(&self.units[defender_idx]);
        let attacker_def = self.units[attacker_idx].defense();
        let defender_attack = self.units[defender_idx].attack();
        
        // Damage formula: attack * 30 / (30 + defense)
        let damage_to_defender = attacker_attack * 30 / (30 + defender_effective_def);
        let damage_to_attacker = defender_attack * 30 / (30 + attacker_def) / 2; // Counterattack is weaker
        
        // Apply damage
        self.units[defender_idx].hp = self.units[defender_idx].hp.saturating_sub(damage_to_defender);
        self.units[attacker_idx].hp = self.units[attacker_idx].hp.saturating_sub(damage_to_attacker);
        
        // Consume all movement on attack
        self.units[attacker_idx].movement_remaining = 0;
        
        let attacker_hp = self.units[attacker_idx].hp;
        let defender_hp = self.units[defender_idx].hp;
        let defender_pos = (self.units[defender_idx].q, self.units[defender_idx].r);
        let attacker_owner = self.units[attacker_idx].owner_id.clone();
        
        // Remove dead units
        let mut attacker_died = false;
        let mut defender_died = false;
        
        if defender_hp == 0 {
            defender_died = true;
            self.units.remove(defender_idx);
        }
        if attacker_hp == 0 {
            attacker_died = true;
            let idx = self.units.iter().position(|u| u.id == attacker_id).unwrap();
            self.units.remove(idx);
        }
        
        // If defender died, move attacker and check for city capture
        let mut captured_city = None;
        let mut eliminated_player = None;
        let mut attacker_new_q = None;
        let mut attacker_new_r = None;
        
        if defender_died && !attacker_died {
            // Move attacker to defender's position
            if let Some(attacker) = self.units.iter_mut().find(|u| u.id == attacker_id) {
                attacker.q = defender_pos.0;
                attacker.r = defender_pos.1;
                attacker_new_q = Some(defender_pos.0);
                attacker_new_r = Some(defender_pos.1);
            }
            
            // Check for city capture using shared method
            let (cap_city, elim_player) = self.try_capture_city(defender_pos.0, defender_pos.1, &attacker_owner);
            captured_city = cap_city;
            eliminated_player = elim_player;
        }
        
        Ok(CombatOutcome {
            attacker_hp,
            defender_hp,
            damage_to_attacker,
            damage_to_defender,
            attacker_died,
            defender_died,
            attacker_new_q,
            attacker_new_r,
            captured_city,
            eliminated_player,
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CombatOutcome {
    pub attacker_hp: u32,
    pub defender_hp: u32,
    pub damage_to_attacker: u32,
    pub damage_to_defender: u32,
    pub attacker_died: bool,
    pub defender_died: bool,
    pub attacker_new_q: Option<i32>,
    pub attacker_new_r: Option<i32>,
    pub captured_city: Option<City>,
    pub eliminated_player: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MoveOutcome {
    pub movement_remaining: u32,
    pub captured_city: Option<City>,
    pub eliminated_player: Option<String>,
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
    AttackUnit { game_id: String, player_id: String, attacker_id: String, defender_id: String },
    FortifyUnit { game_id: String, player_id: String, unit_id: String },
    BuyUnit { game_id: String, player_id: String, city_id: String, unit_type: String },
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
    TurnChanged { current_turn: usize, player_times_ms: Vec<u64>, player_gold: Vec<u64>, units: Vec<Unit>, cities: Vec<City> },
    TimeTick { player_index: usize, remaining_ms: u64 },
    UnitMoved { unit_id: String, to_q: i32, to_r: i32, movement_remaining: u32 },
    CombatResult {
        attacker_id: String,
        defender_id: String,
        attacker_hp: u32,
        defender_hp: u32,
        damage_to_attacker: u32,
        damage_to_defender: u32,
        attacker_died: bool,
        defender_died: bool,
        attacker_new_q: Option<i32>,
        attacker_new_r: Option<i32>,
    },
    PlayerEliminated { player_id: String, conquerer_id: String },
    CitiesCaptured { cities: Vec<City> },
    GameOver { winner_id: String },
    UnitFortified { unit_id: String, new_hp: u32 },
    UnitPurchased { unit: Unit, city_id: String, player_gold: u64 },
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
