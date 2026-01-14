use palmietopia_core::{GameSession, ServerMessage};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{broadcast, RwLock};
use tokio::time::{interval, Duration};

pub struct ActiveGame {
    pub game: GameSession,
    pub channel: broadcast::Sender<String>,
}

pub struct GameManager {
    pub active_games: Arc<RwLock<HashMap<String, ActiveGame>>>,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            active_games: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn start_game(&self, mut game: GameSession, channel: broadcast::Sender<String>) {
        let game_id = game.id.clone();
        
        // Set the turn start time
        game.turn_started_at_ms = current_time_ms();

        let active_game = ActiveGame {
            game: game.clone(),
            channel: channel.clone(),
        };

        {
            let mut games = self.active_games.write().await;
            games.insert(game_id.clone(), active_game);
        }

        // Spawn timer task for this game
        let games_ref = Arc::clone(&self.active_games);
        tokio::spawn(async move {
            run_game_timer(game_id, games_ref).await;
        });
    }

    pub async fn end_turn(&self, game_id: &str, player_id: &str) -> Result<GameSession, String> {
        tracing::info!("end_turn called: game_id={}, player_id={}", game_id, player_id);
        
        let mut games = self.active_games.write().await;
        let active_game = games.get_mut(game_id).ok_or_else(|| {
            tracing::error!("Game not found: {}", game_id);
            "Game not found".to_string()
        })?;

        // Verify it's this player's turn
        let current_player = &active_game.game.players[active_game.game.current_turn];
        tracing::info!("Current player: id={}, name={}", current_player.id, current_player.name);
        
        if current_player.id != player_id {
            tracing::error!("Not your turn: expected={}, got={}", current_player.id, player_id);
            return Err(format!("Not your turn (expected {}, got {})", current_player.id, player_id));
        }

        // Calculate time used
        let now = current_time_ms();
        let time_used = now.saturating_sub(active_game.game.turn_started_at_ms);
        tracing::info!("Time used: {}ms", time_used);

        // End turn (subtracts time used, adds increment, advances to next player)
        active_game.game.end_current_turn(time_used);
        active_game.game.turn_started_at_ms = now;

        tracing::info!("Turn ended. New turn: {}, player_times: {:?}", 
            active_game.game.current_turn, active_game.game.player_times_ms);

        // Broadcast turn change to all subscribed clients
        // Visibility is calculated client-side using explored_tiles from GameSession
        let msg = ServerMessage::TurnChanged {
            current_turn: active_game.game.current_turn,
            player_times_ms: active_game.game.player_times_ms.clone(),
            player_gold: active_game.game.player_gold.clone(),
            units: active_game.game.units.clone(),
            cities: active_game.game.cities.clone(),
            explored_tiles: active_game.game.explored_tiles.clone(),
        };
        let _ = active_game.channel.send(serde_json::to_string(&msg).unwrap());

        Ok(active_game.game.clone())
    }

    pub async fn get_game(&self, game_id: &str) -> Option<GameSession> {
        let games = self.active_games.read().await;
        games.get(game_id).map(|g| g.game.clone())
    }

    pub async fn move_unit(&self, game_id: &str, player_id: &str, unit_id: &str, to_q: i32, to_r: i32) -> Result<palmietopia_core::MoveOutcome, String> {
        tracing::info!("move_unit called: game_id={}, player_id={}, unit_id={}", game_id, player_id, unit_id);
        
        let mut games = self.active_games.write().await;
        let active_game = games.get_mut(game_id).ok_or_else(|| {
            tracing::error!("Game not found: {}", game_id);
            "Game not found".to_string()
        })?;

        // Verify it's this player's turn
        let current_player = &active_game.game.players[active_game.game.current_turn];
        if current_player.id != player_id {
            tracing::error!("Not your turn: expected={}, got={}", current_player.id, player_id);
            return Err("Not your turn".to_string());
        }

        // Verify the unit belongs to the player
        let unit = active_game.game.units.iter().find(|u| u.id == unit_id)
            .ok_or("Unit not found")?;
        if unit.owner_id != player_id {
            return Err("Not your unit".to_string());
        }

        // Perform the move (validates and updates position, may capture city)
        let outcome = active_game.game.move_unit(unit_id, to_q, to_r)?;

        // Broadcast the move to all players (includes updated exploration)
        let msg = ServerMessage::UnitMoved {
            unit_id: unit_id.to_string(),
            to_q,
            to_r,
            movement_remaining: outcome.movement_remaining,
            explored_tiles: active_game.game.explored_tiles.clone(),
        };
        let _ = active_game.channel.send(serde_json::to_string(&msg).unwrap());

        // If a player was eliminated, broadcast that
        if let Some(ref eliminated_id) = outcome.eliminated_player {
            let elim_msg = ServerMessage::PlayerEliminated {
                player_id: eliminated_id.clone(),
                conquerer_id: player_id.to_string(),
            };
            let _ = active_game.channel.send(serde_json::to_string(&elim_msg).unwrap());

            // Broadcast updated cities
            let cities_msg = ServerMessage::CitiesCaptured {
                cities: active_game.game.cities.clone(),
            };
            let _ = active_game.channel.send(serde_json::to_string(&cities_msg).unwrap());
        } else if outcome.captured_city.is_some() {
            // Just a regular city capture (non-capitol)
            let cities_msg = ServerMessage::CitiesCaptured {
                cities: active_game.game.cities.clone(),
            };
            let _ = active_game.channel.send(serde_json::to_string(&cities_msg).unwrap());
        }

        // If game is over, broadcast victory
        if let palmietopia_core::GameStatus::Victory { ref winner_id } = active_game.game.status {
            let victory_msg = ServerMessage::GameOver {
                winner_id: winner_id.clone(),
            };
            let _ = active_game.channel.send(serde_json::to_string(&victory_msg).unwrap());
        }

        Ok(outcome)
    }

    pub async fn attack_unit(&self, game_id: &str, player_id: &str, attacker_id: &str, defender_id: &str) -> Result<palmietopia_core::CombatOutcome, String> {
        tracing::info!("attack_unit called: game_id={}, player_id={}, attacker={}, defender={}", 
            game_id, player_id, attacker_id, defender_id);
        
        let mut games = self.active_games.write().await;
        let active_game = games.get_mut(game_id).ok_or_else(|| {
            tracing::error!("Game not found: {}", game_id);
            "Game not found".to_string()
        })?;

        // Verify it's this player's turn
        let current_player = &active_game.game.players[active_game.game.current_turn];
        if current_player.id != player_id {
            return Err("Not your turn".to_string());
        }

        // Verify the attacker belongs to the player
        let attacker = active_game.game.units.iter().find(|u| u.id == attacker_id)
            .ok_or("Attacker not found")?;
        if attacker.owner_id != player_id {
            return Err("Not your unit".to_string());
        }

        // Resolve combat
        let outcome = active_game.game.resolve_combat(attacker_id, defender_id)?;

        // Broadcast combat result
        let msg = ServerMessage::CombatResult {
            attacker_id: attacker_id.to_string(),
            defender_id: defender_id.to_string(),
            attacker_hp: outcome.attacker_hp,
            defender_hp: outcome.defender_hp,
            damage_to_attacker: outcome.damage_to_attacker,
            damage_to_defender: outcome.damage_to_defender,
            attacker_died: outcome.attacker_died,
            defender_died: outcome.defender_died,
            attacker_new_q: outcome.attacker_new_q,
            attacker_new_r: outcome.attacker_new_r,
        };
        let _ = active_game.channel.send(serde_json::to_string(&msg).unwrap());

        // If a player was eliminated, broadcast that too
        if let Some(ref eliminated_id) = outcome.eliminated_player {
            let elim_msg = ServerMessage::PlayerEliminated {
                player_id: eliminated_id.clone(),
                conquerer_id: player_id.to_string(),
            };
            let _ = active_game.channel.send(serde_json::to_string(&elim_msg).unwrap());

            // Broadcast updated cities
            let cities_msg = ServerMessage::CitiesCaptured {
                cities: active_game.game.cities.clone(),
            };
            let _ = active_game.channel.send(serde_json::to_string(&cities_msg).unwrap());
        }

        // If game is over, broadcast victory
        if let palmietopia_core::GameStatus::Victory { ref winner_id } = active_game.game.status {
            let victory_msg = ServerMessage::GameOver {
                winner_id: winner_id.clone(),
            };
            let _ = active_game.channel.send(serde_json::to_string(&victory_msg).unwrap());
        }

        Ok(outcome)
    }

    pub async fn fortify_unit(&self, game_id: &str, player_id: &str, unit_id: &str) -> Result<u32, String> {
        tracing::info!("fortify_unit called: game_id={}, player_id={}, unit_id={}", game_id, player_id, unit_id);
        
        let mut games = self.active_games.write().await;
        let active_game = games.get_mut(game_id).ok_or_else(|| {
            tracing::error!("Game not found: {}", game_id);
            "Game not found".to_string()
        })?;

        // Verify it's this player's turn
        let current_player = &active_game.game.players[active_game.game.current_turn];
        if current_player.id != player_id {
            return Err("Not your turn".to_string());
        }

        // Verify the unit belongs to the player
        let unit = active_game.game.units.iter().find(|u| u.id == unit_id)
            .ok_or("Unit not found")?;
        if unit.owner_id != player_id {
            return Err("Not your unit".to_string());
        }

        // Perform fortify
        let new_hp = active_game.game.fortify_unit(unit_id)?;

        // Broadcast the fortify to all players
        let msg = ServerMessage::UnitFortified {
            unit_id: unit_id.to_string(),
            new_hp,
        };
        let _ = active_game.channel.send(serde_json::to_string(&msg).unwrap());

        Ok(new_hp)
    }

    pub async fn buy_unit(&self, game_id: &str, player_id: &str, city_id: &str, unit_type: palmietopia_core::UnitType) -> Result<(palmietopia_core::Unit, u64), String> {
        tracing::info!("buy_unit called: game_id={}, player_id={}, city_id={}, unit_type={:?}", 
            game_id, player_id, city_id, unit_type);
        
        let mut games = self.active_games.write().await;
        let active_game = games.get_mut(game_id).ok_or_else(|| {
            tracing::error!("Game not found: {}", game_id);
            "Game not found".to_string()
        })?;

        // Verify it's this player's turn
        let current_player = &active_game.game.players[active_game.game.current_turn];
        if current_player.id != player_id {
            return Err("Not your turn".to_string());
        }

        // Buy the unit
        let unit = active_game.game.buy_unit(player_id, city_id, unit_type)?;
        
        // Get player's new gold amount
        let player_idx = active_game.game.players.iter().position(|p| p.id == player_id).unwrap();
        let player_gold = active_game.game.player_gold[player_idx];

        // Broadcast the purchase to all players
        let msg = ServerMessage::UnitPurchased {
            unit: unit.clone(),
            city_id: city_id.to_string(),
            player_gold,
        };
        let _ = active_game.channel.send(serde_json::to_string(&msg).unwrap());

        Ok((unit, player_gold))
    }

    pub async fn get_channel_async(&self, game_id: &str) -> Option<broadcast::Sender<String>> {
        let games = self.active_games.read().await;
        games.get(game_id).map(|g| g.channel.clone())
    }
}

impl Default for GameManager {
    fn default() -> Self {
        Self::new()
    }
}

pub fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

async fn run_game_timer(game_id: String, games: Arc<RwLock<HashMap<String, ActiveGame>>>) {
    let mut tick_interval = interval(Duration::from_secs(1));

    loop {
        tick_interval.tick().await;

        {
            let mut games_lock = games.write().await;
            if let Some(active_game) = games_lock.get_mut(&game_id) {
                // Stop timer if game is over
                if let palmietopia_core::GameStatus::Victory { .. } = active_game.game.status {
                    tracing::info!("Game {} ended (victory), stopping timer and cleaning up", game_id);
                    games_lock.remove(&game_id);
                    break;
                }

                let now = current_time_ms();
                let elapsed = now.saturating_sub(active_game.game.turn_started_at_ms);
                let current_player_time = active_game.game.current_player_time();
                let remaining = current_player_time.saturating_sub(elapsed);

                // Broadcast time tick for current player
                let tick_msg = ServerMessage::TimeTick { 
                    player_index: active_game.game.current_turn,
                    remaining_ms: remaining,
                };
                let _ = active_game.channel.send(serde_json::to_string(&tick_msg).unwrap());

                // Auto-end turn if time runs out
                if remaining == 0 {
                    tracing::info!("Auto-ending turn for player {} (time ran out)", active_game.game.current_turn);
                    
                    // End turn with full time used (they ran out)
                    active_game.game.end_current_turn(current_player_time);
                    active_game.game.turn_started_at_ms = now;

                    let turn_msg = ServerMessage::TurnChanged {
                        current_turn: active_game.game.current_turn,
                        player_times_ms: active_game.game.player_times_ms.clone(),
                        player_gold: active_game.game.player_gold.clone(),
                        units: active_game.game.units.clone(),
                        cities: active_game.game.cities.clone(),
                        explored_tiles: active_game.game.explored_tiles.clone(),
                    };
                    let _ = active_game.channel.send(serde_json::to_string(&turn_msg).unwrap());
                }
            } else {
                // Game no longer exists, stop the timer
                break;
            }
        }
    }
}
