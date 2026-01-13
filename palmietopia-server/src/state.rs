use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::store::GameStore;

pub type Tx = broadcast::Sender<String>;

#[allow(dead_code)]
pub struct PlayerConnection {
    pub player_id: String,
    pub lobby_id: Option<String>,
}

pub struct AppState {
    pub store: Arc<dyn GameStore>,
    pub connections: RwLock<HashMap<String, PlayerConnection>>,
    pub lobby_channels: RwLock<HashMap<String, Tx>>,
}

impl AppState {
    pub fn new(store: Arc<dyn GameStore>) -> Self {
        Self {
            store,
            connections: RwLock::new(HashMap::new()),
            lobby_channels: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get_or_create_lobby_channel(&self, lobby_id: &str) -> Tx {
        let mut channels = self.lobby_channels.write().await;
        if let Some(tx) = channels.get(lobby_id) {
            tx.clone()
        } else {
            let (tx, _) = broadcast::channel(100);
            channels.insert(lobby_id.to_string(), tx.clone());
            tx
        }
    }

    pub async fn remove_lobby_channel(&self, lobby_id: &str) {
        let mut channels = self.lobby_channels.write().await;
        channels.remove(lobby_id);
    }
}
