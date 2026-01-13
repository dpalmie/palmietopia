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
        let msg = ServerMessage::TurnChanged {
            current_turn: active_game.game.current_turn,
            player_times_ms: active_game.game.player_times_ms.clone(),
        };
        let _ = active_game.channel.send(serde_json::to_string(&msg).unwrap());

        Ok(active_game.game.clone())
    }

    pub async fn get_game(&self, game_id: &str) -> Option<GameSession> {
        let games = self.active_games.read().await;
        games.get(game_id).map(|g| g.game.clone())
    }

    pub fn get_channel(&self, game_id: &str) -> Option<broadcast::Sender<String>> {
        // This needs to be sync for use in ws handler
        // We'll handle this differently
        None
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
