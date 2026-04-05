use serde::{Deserialize, Serialize};

use crate::game::types::{Color, GameResult, Reserve, Role, Square};

/// Messages sent from client to server.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Make a normal move (piece from one square to another).
    Move {
        from: Square,
        to: Square,
        #[serde(default)]
        promotion: Option<Role>,
    },
    /// Drop a piece from reserve onto the board.
    Drop { role: Role, to: Square },
    /// Ping for keepalive.
    Ping,
}

/// Messages sent from server to client.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Game has started, here's the initial state.
    GameStart { color: Color, fen: String },
    /// A move was made on the board.
    Move {
        color: Color,
        from: Square,
        to: Square,
        #[serde(skip_serializing_if = "Option::is_none")]
        promotion: Option<Role>,
        fen: String,
    },
    /// A piece was dropped from reserve.
    Drop {
        color: Color,
        role: Role,
        to: Square,
        fen: String,
    },
    /// Reserve state updated (after a capture on the partner's board).
    ReserveUpdate { white: Reserve, black: Reserve },
    /// Game is over.
    GameOver { result: GameResult },
    /// An error occurred processing the client's message.
    Error { message: String },
    /// Pong response to client ping.
    Pong,
}
