use std::sync::Arc;

use axum::Router;
use axum::routing::get;

use crate::AppState;
use crate::ws::handler::ws_handler;

/// Builds the full Axum router with all routes.
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ws", get(ws_handler))
        .route("/health", get(health_check))
        .with_state(state)
}

async fn health_check() -> &'static str {
    "ok"
}
