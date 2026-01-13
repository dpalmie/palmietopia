use async_trait::async_trait;
use palmietopia_core::{GameSession, Lobby};

pub mod memory;

pub type StoreResult<T> = Result<T, StoreError>;

#[derive(Debug)]
#[allow(dead_code)]
pub enum StoreError {
    NotFound,
    AlreadyExists,
    Internal(String),
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::NotFound => write!(f, "Not found"),
            StoreError::AlreadyExists => write!(f, "Already exists"),
            StoreError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for StoreError {}

#[async_trait]
pub trait GameStore: Send + Sync {
    // Lobby operations
    async fn create_lobby(&self, lobby: Lobby) -> StoreResult<String>;
    async fn get_lobby(&self, id: &str) -> StoreResult<Option<Lobby>>;
    async fn list_lobbies(&self) -> StoreResult<Vec<Lobby>>;
    async fn update_lobby(&self, lobby: Lobby) -> StoreResult<()>;
    async fn delete_lobby(&self, id: &str) -> StoreResult<()>;

    // Game operations
    async fn save_game(&self, game: GameSession) -> StoreResult<()>;
    async fn load_game(&self, id: &str) -> StoreResult<Option<GameSession>>;
}
