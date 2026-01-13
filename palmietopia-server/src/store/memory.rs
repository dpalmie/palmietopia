use async_trait::async_trait;
use palmietopia_core::{GameSession, Lobby};
use std::collections::HashMap;
use std::sync::RwLock;

use super::{GameStore, StoreResult};

pub struct InMemoryStore {
    lobbies: RwLock<HashMap<String, Lobby>>,
    games: RwLock<HashMap<String, GameSession>>,
}

impl InMemoryStore {
    pub fn new() -> Self {
        Self {
            lobbies: RwLock::new(HashMap::new()),
            games: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for InMemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GameStore for InMemoryStore {
    async fn create_lobby(&self, lobby: Lobby) -> StoreResult<String> {
        let id = lobby.id.clone();
        let mut lobbies = self.lobbies.write().unwrap();
        lobbies.insert(id.clone(), lobby);
        Ok(id)
    }

    async fn get_lobby(&self, id: &str) -> StoreResult<Option<Lobby>> {
        let lobbies = self.lobbies.read().unwrap();
        Ok(lobbies.get(id).cloned())
    }

    async fn list_lobbies(&self) -> StoreResult<Vec<Lobby>> {
        let lobbies = self.lobbies.read().unwrap();
        Ok(lobbies.values().cloned().collect())
    }

    async fn update_lobby(&self, lobby: Lobby) -> StoreResult<()> {
        let mut lobbies = self.lobbies.write().unwrap();
        lobbies.insert(lobby.id.clone(), lobby);
        Ok(())
    }

    async fn delete_lobby(&self, id: &str) -> StoreResult<()> {
        let mut lobbies = self.lobbies.write().unwrap();
        lobbies.remove(id);
        Ok(())
    }

    async fn save_game(&self, game: GameSession) -> StoreResult<()> {
        let mut games = self.games.write().unwrap();
        games.insert(game.id.clone(), game);
        Ok(())
    }

    async fn load_game(&self, id: &str) -> StoreResult<Option<GameSession>> {
        let games = self.games.read().unwrap();
        Ok(games.get(id).cloned())
    }
}
