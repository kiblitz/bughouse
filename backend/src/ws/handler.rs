use std::sync::Arc;

use axum::extract::ws::{Message, WebSocket};
use axum::extract::{State, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures_util::{SinkExt, StreamExt};
use tokio::sync::mpsc;
use tracing::{info, warn};

use crate::game::types::{Color, GameMove, GameResult, Termination};
use crate::ws::messages::{ClientMessage, ServerMessage};
use crate::{AppState, CLIENT_CHANNEL_CAPACITY, MAX_PLAYERS};

/// HTTP handler that upgrades the connection to a WebSocket.
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_connection(socket, state))
}

/// RAII guard that removes a player's sender from the DashMap on drop.
/// Ensures cleanup even if the connection handler panics.
struct ConnectionGuard {
    color: Color,
    state: Arc<AppState>,
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        self.state.senders.remove(&self.color);
        self.state
            .player_count
            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        info!("Player {:?} disconnected (cleanup)", self.color);
    }
}

/// Per-connection WebSocket handler.
///
/// For Phase 1, this implements a simple two-player game on a single board.
/// Players are assigned colors in the order they connect (first = white, second = black).
/// Connections beyond MAX_PLAYERS are rejected.
async fn handle_connection(socket: WebSocket, state: Arc<AppState>) {
    let (mut ws_sender, mut ws_receiver) = socket.split();

    // Reject connections beyond the max player count.
    let player_count = state
        .player_count
        .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    if player_count >= MAX_PLAYERS {
        state
            .player_count
            .fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
        let msg = ServerMessage::Error {
            message: "Game is full".to_string(),
        };
        let text = serde_json::to_string(&msg).expect("ServerMessage serializes to JSON");
        let _ = ws_sender.send(Message::Text(text.into())).await;
        return;
    }

    let color = if player_count == 0 {
        Color::White
    } else {
        Color::Black
    };

    info!("Player connected as {:?}", color);

    // Bounded channel — applies backpressure if a client stalls.
    let (tx, mut rx) = mpsc::channel::<ServerMessage>(CLIENT_CHANNEL_CAPACITY);

    // Register this sender.
    state.senders.insert(color, tx.clone());

    // RAII guard ensures senders entry is removed even on panic.
    let _guard = ConnectionGuard {
        color,
        state: Arc::clone(&state),
    };

    // Send initial game state to this player.
    {
        let board = state.board.lock().await;
        let msg = ServerMessage::GameStart {
            color,
            fen: board.fen(),
        };
        let _ = tx.send(msg).await;
    }

    // Spawn task to forward outgoing messages from channel to WebSocket.
    let send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            let text = serde_json::to_string(&msg).expect("ServerMessage serializes to JSON");
            if ws_sender.send(Message::Text(text.into())).await.is_err() {
                break;
            }
        }
    });

    // Process incoming messages from the client.
    while let Some(Ok(msg)) = ws_receiver.next().await {
        match msg {
            Message::Text(text) => {
                let client_msg: ClientMessage = match serde_json::from_str(&text) {
                    Ok(m) => m,
                    Err(e) => {
                        warn!("Invalid message from {:?}: {}", color, e);
                        let _ = tx.try_send(ServerMessage::Error {
                            message: format!("Invalid message: {}", e),
                        });
                        continue;
                    }
                };
                handle_client_message(color, client_msg, &state, &tx).await;
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    // Drop the sender to signal the send_task to exit cleanly.
    drop(tx);
    let _ = send_task.await;
    // _guard drops here, cleaning up senders and player_count.
}

async fn handle_client_message(
    color: Color,
    msg: ClientMessage,
    state: &AppState,
    tx: &mpsc::Sender<ServerMessage>,
) {
    match msg {
        ClientMessage::Ping => {
            let _ = tx.try_send(ServerMessage::Pong);
        }
        ClientMessage::Move {
            from,
            to,
            promotion,
        } => {
            let game_move = GameMove::Normal {
                from,
                to,
                promotion,
            };
            handle_game_move(color, game_move, state).await;
        }
        ClientMessage::Drop { role, to } => {
            let game_move = GameMove::Drop { role, to };
            handle_game_move(color, game_move, state).await;
        }
    }
}

async fn handle_game_move(color: Color, game_move: GameMove, state: &AppState) {
    // Compute move result under the lock, then drop before broadcasting.
    let move_result = {
        let mut board = state.board.lock().await;

        // Verify it's this player's turn.
        if board.turn() != color {
            if let Some(sender) = state.senders.get(&color) {
                let _ = sender.try_send(ServerMessage::Error {
                    message: "Not your turn".to_string(),
                });
            }
            return;
        }

        match board.make_move(&game_move) {
            Ok(_captured) => {
                let fen = board.fen();
                let is_checkmate = board.is_checkmate();
                let is_stalemate = board.is_stalemate();
                Ok((fen, is_checkmate, is_stalemate))
            }
            Err(e) => Err(e.to_string()),
        }
    };
    // Lock is dropped here.

    match move_result {
        Ok((fen, is_checkmate, is_stalemate)) => {
            let server_msg = match game_move {
                GameMove::Normal {
                    from,
                    to,
                    promotion,
                } => ServerMessage::Move {
                    color,
                    from,
                    to,
                    promotion,
                    fen,
                },
                GameMove::Drop { role, to } => ServerMessage::Drop {
                    color,
                    role,
                    to,
                    fen,
                },
            };

            broadcast(state, &server_msg);

            // Check for game over.
            if is_checkmate {
                // In Phase 1 (single board), winner is the color that just moved.
                let result = GameResult {
                    winner: Some(color),
                    termination: Termination::Checkmate,
                };
                broadcast(state, &ServerMessage::GameOver { result });
            } else if is_stalemate {
                let result = GameResult {
                    winner: None,
                    termination: Termination::Draw,
                };
                broadcast(state, &ServerMessage::GameOver { result });
            }
        }
        Err(e) => {
            if let Some(sender) = state.senders.get(&color) {
                let _ = sender.try_send(ServerMessage::Error { message: e });
            }
        }
    }
}

fn broadcast(state: &AppState, msg: &ServerMessage) {
    for entry in state.senders.iter() {
        let _ = entry.value().try_send(msg.clone());
    }
}
