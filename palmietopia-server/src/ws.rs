use axum::extract::ws::{Message, WebSocket};
use futures::{SinkExt, StreamExt};
use palmietopia_core::{
    ClientMessage, GameSession, Lobby, LobbyStatus, Player, PlayerColor, ServerMessage,
};
use std::sync::Arc;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::state::AppState;

pub async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    let player_id = Uuid::new_v4().to_string();
    let mut current_lobby_id: Option<String> = None;
    let mut lobby_rx: Option<broadcast::Receiver<String>> = None;

    let mut current_game_id: Option<String> = None;

    // Register connection
    {
        let mut connections = state.connections.write().await;
        connections.insert(
            player_id.clone(),
            crate::state::PlayerConnection {
                player_id: player_id.clone(),
                lobby_id: None,
                game_id: None,
            },
        );
    }

    loop {
        tokio::select! {
            // Handle incoming messages from client
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ClientMessage>(&text) {
                            Ok(client_msg) => {
                                let response = handle_client_message(
                                    client_msg,
                                    &player_id,
                                    &mut current_lobby_id,
                                    &mut current_game_id,
                                    &mut lobby_rx,
                                    &state,
                                ).await;

                                if let Some(msg) = response {
                                    let json = serde_json::to_string(&msg).unwrap();
                                    if sender.send(Message::Text(json.into())).await.is_err() {
                                        break;
                                    }
                                }
                            }
                            Err(e) => {
                                let error = ServerMessage::Error {
                                    message: format!("Invalid message format: {}", e),
                                };
                                let json = serde_json::to_string(&error).unwrap();
                                let _ = sender.send(Message::Text(json.into())).await;
                            }
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => break,
                    _ => {}
                }
            }

            // Handle broadcast messages from lobby
            broadcast_msg = async {
                if let Some(ref mut rx) = lobby_rx {
                    rx.recv().await.ok()
                } else {
                    std::future::pending::<Option<String>>().await
                }
            } => {
                if let Some(msg) = broadcast_msg {
                    if sender.send(Message::Text(msg.into())).await.is_err() {
                        break;
                    }
                }
            }
        }
    }

    // Cleanup on disconnect
    handle_disconnect(&player_id, &current_lobby_id, &state).await;
}

async fn handle_client_message(
    msg: ClientMessage,
    player_id: &str,
    current_lobby_id: &mut Option<String>,
    current_game_id: &mut Option<String>,
    lobby_rx: &mut Option<broadcast::Receiver<String>>,
    state: &Arc<AppState>,
) -> Option<ServerMessage> {
    match msg {
        ClientMessage::ListLobbies => {
            let lobbies = state.store.list_lobbies().await.unwrap_or_default();
            let visible_lobbies: Vec<Lobby> = lobbies
                .into_iter()
                .filter(|l| l.status == LobbyStatus::Waiting)
                .collect();
            Some(ServerMessage::LobbyList {
                lobbies: visible_lobbies,
            })
        }

        ClientMessage::CreateLobby {
            player_name,
            map_size,
        } => {
            // Prevent creating if already in a lobby
            if current_lobby_id.is_some() {
                return Some(ServerMessage::Error {
                    message: "Already in a lobby. Leave first before creating a new one.".to_string(),
                });
            }

            let lobby_id = Uuid::new_v4().to_string();
            let player = Player {
                id: player_id.to_string(),
                name: player_name,
                color: PlayerColor::Red,
            };

            let lobby = Lobby::new(lobby_id.clone(), player, map_size);
            if let Err(e) = state.store.create_lobby(lobby.clone()).await {
                return Some(ServerMessage::Error {
                    message: format!("Failed to create lobby: {}", e),
                });
            }

            // Subscribe to lobby channel
            let tx = state.get_or_create_lobby_channel(&lobby_id).await;
            *lobby_rx = Some(tx.subscribe());
            *current_lobby_id = Some(lobby_id.clone());

            // Update connection state
            {
                let mut connections = state.connections.write().await;
                if let Some(conn) = connections.get_mut(player_id) {
                    conn.lobby_id = Some(lobby_id.clone());
                }
            }

            // Broadcast lobby state to the creator (so they see the lobby room)
            let lobby_update = ServerMessage::LobbyUpdated { lobby: lobby.clone() };
            let _ = tx.send(serde_json::to_string(&lobby_update).unwrap());

            Some(ServerMessage::LobbyCreated {
                lobby_id,
                player_id: player_id.to_string(),
            })
        }

        ClientMessage::JoinLobby {
            lobby_id,
            player_name,
        } => {
            // Prevent joining if already in a lobby
            if current_lobby_id.is_some() {
                return Some(ServerMessage::Error {
                    message: "Already in a lobby. Leave first before joining another.".to_string(),
                });
            }

            let lobby = match state.store.get_lobby(&lobby_id).await {
                Ok(Some(l)) => l,
                Ok(None) => {
                    return Some(ServerMessage::Error {
                        message: "Lobby not found".to_string(),
                    });
                }
                Err(e) => {
                    return Some(ServerMessage::Error {
                        message: format!("Failed to get lobby: {}", e),
                    });
                }
            };

            if !lobby.can_join() {
                return Some(ServerMessage::Error {
                    message: "Cannot join this lobby".to_string(),
                });
            }

            // Prevent joining a lobby you're already in
            if lobby.players.iter().any(|p| p.id == player_id) {
                return Some(ServerMessage::Error {
                    message: "You are already in this lobby".to_string(),
                });
            }

            let player = Player {
                id: player_id.to_string(),
                name: player_name,
                color: PlayerColor::from_index(lobby.players.len()),
            };

            let mut updated_lobby = lobby;
            updated_lobby.players.push(player);

            if let Err(e) = state.store.update_lobby(updated_lobby.clone()).await {
                return Some(ServerMessage::Error {
                    message: format!("Failed to join lobby: {}", e),
                });
            }

            // Subscribe to lobby channel
            let tx = state.get_or_create_lobby_channel(&lobby_id).await;
            *lobby_rx = Some(tx.subscribe());
            *current_lobby_id = Some(lobby_id.clone());

            // Update connection state
            {
                let mut connections = state.connections.write().await;
                if let Some(conn) = connections.get_mut(player_id) {
                    conn.lobby_id = Some(lobby_id);
                }
            }

            // Broadcast updated lobby to all players
            let update_msg = ServerMessage::LobbyUpdated {
                lobby: updated_lobby.clone(),
            };
            let _ = tx.send(serde_json::to_string(&update_msg).unwrap());

            Some(ServerMessage::JoinedLobby {
                lobby: updated_lobby,
                player_id: player_id.to_string(),
            })
        }

        ClientMessage::LeaveLobby => {
            if let Some(lobby_id) = current_lobby_id.take() {
                leave_lobby(player_id, &lobby_id, state).await;
                *lobby_rx = None;

                // Update connection state
                {
                    let mut connections = state.connections.write().await;
                    if let Some(conn) = connections.get_mut(player_id) {
                        conn.lobby_id = None;
                    }
                }
            }
            None
        }

        ClientMessage::StartGame => {
            let lobby_id = match current_lobby_id {
                Some(id) => id.clone(),
                None => {
                    return Some(ServerMessage::Error {
                        message: "Not in a lobby".to_string(),
                    });
                }
            };

            let lobby = match state.store.get_lobby(&lobby_id).await {
                Ok(Some(l)) => l,
                _ => {
                    return Some(ServerMessage::Error {
                        message: "Lobby not found".to_string(),
                    });
                }
            };

            if lobby.host_id != player_id {
                return Some(ServerMessage::Error {
                    message: "Only the host can start the game".to_string(),
                });
            }

            if !lobby.can_start() {
                return Some(ServerMessage::Error {
                    message: "Need at least 2 players to start".to_string(),
                });
            }

            // Create game session with timestamp
            let mut game = GameSession::from_lobby(&lobby);
            game.turn_started_at_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            // Update lobby status
            let mut updated_lobby = lobby;
            updated_lobby.status = LobbyStatus::InGame;
            let _ = state.store.update_lobby(updated_lobby).await;

            // Save game
            let _ = state.store.save_game(game.clone()).await;

            // Get channel and start the game with timer
            let tx = state.get_or_create_lobby_channel(&lobby_id).await;
            state.game_manager.start_game(game.clone(), tx.clone()).await;

            // Set current game ID
            *current_game_id = Some(game.id.clone());

            // Broadcast game start to all players
            let start_msg = ServerMessage::GameStarted { game: game.clone() };
            let _ = tx.send(serde_json::to_string(&start_msg).unwrap());

            Some(ServerMessage::GameStarted { game })
        }

        ClientMessage::EndTurn { game_id, player_id: msg_player_id } => {
            tracing::info!("EndTurn received: game_id={}, player_id={}", game_id, msg_player_id);
            match state.game_manager.end_turn(&game_id, &msg_player_id).await {
                Ok(game) => {
                    tracing::info!("EndTurn succeeded");
                    // Return TurnChanged directly (broadcast also sent to subscribed clients)
                    Some(ServerMessage::TurnChanged {
                        current_turn: game.current_turn,
                        player_times_ms: game.player_times_ms.clone(),
                        units: game.units.clone(),
                    })
                }
                Err(e) => {
                    tracing::error!("EndTurn failed: {}", e);
                    Some(ServerMessage::Error { message: e })
                }
            }
        }

        ClientMessage::RejoinGame { game_id, player_id: msg_player_id } => {
            tracing::info!("RejoinGame received: game_id={}, player_id={}", game_id, msg_player_id);
            
            // Get the game
            let game = match state.game_manager.get_game(&game_id).await {
                Some(g) => g,
                None => {
                    return Some(ServerMessage::Error {
                        message: "Game not found".to_string(),
                    });
                }
            };

            // Verify player is in this game
            if !game.players.iter().any(|p| p.id == msg_player_id) {
                return Some(ServerMessage::Error {
                    message: "You are not in this game".to_string(),
                });
            }

            // Subscribe to game's broadcast channel
            if let Some(tx) = state.game_manager.get_channel_async(&game_id).await {
                *lobby_rx = Some(tx.subscribe());
                *current_game_id = Some(game_id.clone());
                tracing::info!("Player {} rejoined game {}", msg_player_id, game_id);
            }

            Some(ServerMessage::GameRejoined { game })
        }

        ClientMessage::MoveUnit { game_id, player_id: msg_player_id, unit_id, to_q, to_r } => {
            tracing::info!("MoveUnit received: game_id={}, player_id={}, unit_id={}, to=({},{})", 
                game_id, msg_player_id, unit_id, to_q, to_r);
            
            match state.game_manager.move_unit(&game_id, &msg_player_id, &unit_id, to_q, to_r).await {
                Ok(outcome) => {
                    tracing::info!("MoveUnit succeeded, movement_remaining={}", outcome.movement_remaining);
                    // Server already broadcasts via channel, return message for this client
                    Some(ServerMessage::UnitMoved {
                        unit_id,
                        to_q,
                        to_r,
                        movement_remaining: outcome.movement_remaining,
                    })
                }
                Err(e) => {
                    tracing::error!("MoveUnit failed: {}", e);
                    Some(ServerMessage::Error { message: e })
                }
            }
        }

        ClientMessage::AttackUnit { game_id, player_id: msg_player_id, attacker_id, defender_id } => {
            tracing::info!("AttackUnit received: game_id={}, attacker={}, defender={}", 
                game_id, attacker_id, defender_id);
            
            match state.game_manager.attack_unit(&game_id, &msg_player_id, &attacker_id, &defender_id).await {
                Ok(outcome) => {
                    tracing::info!("AttackUnit succeeded: {:?}", outcome);
                    Some(ServerMessage::CombatResult {
                        attacker_id,
                        defender_id,
                        attacker_hp: outcome.attacker_hp,
                        defender_hp: outcome.defender_hp,
                        damage_to_attacker: outcome.damage_to_attacker,
                        damage_to_defender: outcome.damage_to_defender,
                        attacker_died: outcome.attacker_died,
                        defender_died: outcome.defender_died,
                    })
                }
                Err(e) => {
                    tracing::error!("AttackUnit failed: {}", e);
                    Some(ServerMessage::Error { message: e })
                }
            }
        }

        ClientMessage::FortifyUnit { game_id, player_id: msg_player_id, unit_id } => {
            tracing::info!("FortifyUnit received: game_id={}, player_id={}, unit_id={}", 
                game_id, msg_player_id, unit_id);
            
            match state.game_manager.fortify_unit(&game_id, &msg_player_id, &unit_id).await {
                Ok(new_hp) => {
                    tracing::info!("FortifyUnit succeeded, new_hp={}", new_hp);
                    Some(ServerMessage::UnitFortified { unit_id, new_hp })
                }
                Err(e) => {
                    tracing::error!("FortifyUnit failed: {}", e);
                    Some(ServerMessage::Error { message: e })
                }
            }
        }
    }
}

async fn leave_lobby(player_id: &str, lobby_id: &str, state: &Arc<AppState>) {
    let lobby = match state.store.get_lobby(lobby_id).await {
        Ok(Some(l)) => l,
        _ => return,
    };

    let mut updated_lobby = lobby.clone();
    updated_lobby.players.retain(|p| p.id != player_id);

    if updated_lobby.players.is_empty() {
        // Delete empty lobby
        let _ = state.store.delete_lobby(lobby_id).await;
        state.remove_lobby_channel(lobby_id).await;
    } else {
        // If host left, assign new host
        if updated_lobby.host_id == player_id {
            updated_lobby.host_id = updated_lobby.players[0].id.clone();
        }
        let _ = state.store.update_lobby(updated_lobby.clone()).await;

        // Broadcast update
        let tx = state.get_or_create_lobby_channel(lobby_id).await;
        let update_msg = ServerMessage::LobbyUpdated {
            lobby: updated_lobby,
        };
        let _ = tx.send(serde_json::to_string(&update_msg).unwrap());

        let leave_msg = ServerMessage::PlayerLeft {
            player_id: player_id.to_string(),
        };
        let _ = tx.send(serde_json::to_string(&leave_msg).unwrap());
    }
}

async fn handle_disconnect(player_id: &str, current_lobby_id: &Option<String>, state: &Arc<AppState>) {
    // Remove from connections
    {
        let mut connections = state.connections.write().await;
        connections.remove(player_id);
    }

    // Leave lobby if in one
    if let Some(lobby_id) = current_lobby_id {
        leave_lobby(player_id, lobby_id, state).await;
    }
}
