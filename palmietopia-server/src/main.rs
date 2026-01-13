mod game;
mod state;
mod store;
mod ws;

use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use palmietopia_core::Lobby;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber;

use state::AppState;
use store::memory::InMemoryStore;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let store = Arc::new(InMemoryStore::new());
    let app_state = Arc::new(AppState::new(store));

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/ws", get(ws_handler))
        .route("/api/lobbies", get(list_lobbies))
        .route("/health", get(health_check))
        .layer(cors)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    tracing::info!("Server running on http://0.0.0.0:3001");
    axum::serve(listener, app).await.unwrap();
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| ws::handle_socket(socket, state))
}

async fn list_lobbies(State(state): State<Arc<AppState>>) -> Json<Vec<Lobby>> {
    let lobbies = state.store.list_lobbies().await.unwrap_or_default();
    let visible: Vec<Lobby> = lobbies
        .into_iter()
        .filter(|l| l.status == palmietopia_core::LobbyStatus::Waiting)
        .collect();
    Json(visible)
}

async fn health_check() -> &'static str {
    "OK"
}
