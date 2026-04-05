use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

use dashmap::DashMap;
use tokio::sync::{Mutex, mpsc};
use tower_http::cors::{Any, CorsLayer};
use tracing::info;
use tracing_subscriber::EnvFilter;

mod api;
mod game;
mod ws;

use game::board::Board;
use game::types::Color;
use ws::messages::ServerMessage;

const DEFAULT_LISTEN_ADDR: &str = "0.0.0.0:3000";
const MAX_PLAYERS: usize = 2;
/// Channel buffer size per client — bounds memory if a client stalls.
const CLIENT_CHANNEL_CAPACITY: usize = 64;

/// Shared application state.
///
/// For Phase 1, this holds a single board and connected player senders.
/// Phase 2+ will replace this with room-based game management.
pub struct AppState {
    pub board: Mutex<Board>,
    pub senders: DashMap<Color, mpsc::Sender<ServerMessage>>,
    pub player_count: AtomicUsize,
}

#[tokio::main]
async fn main() {
    // Initialize tracing/logging.
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let state = Arc::new(AppState {
        board: Mutex::new(Board::new()),
        senders: DashMap::new(),
        player_count: AtomicUsize::new(0),
    });

    // CORS: allow all origins in development. Phase 3+ will restrict to the
    // GitHub Pages domain.
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = api::routes::build_router(state).layer(cors);

    let addr = std::env::var("LISTEN_ADDR").unwrap_or_else(|_| DEFAULT_LISTEN_ADDR.to_string());
    info!("Bughouse server listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind listen address");
    axum::serve(listener, app)
        .await
        .expect("server exited with error");
}
