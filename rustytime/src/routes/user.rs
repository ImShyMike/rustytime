use crate::state::AppState;
use axum::{
    Router,
    routing::{get, post},
};

use crate::handlers::user::{create_heartbeats, get_statusbar_today};

pub fn id_routes() -> Router<AppState> {
    Router::new()
        // WakaTime compatibility routes
        .route("/heartbeats", post(create_heartbeats))
        .route("/heartbeats.bulk", post(create_heartbeats))
        .route("/statusbar/today", get(get_statusbar_today))
}

pub fn user_routes() -> Router<AppState> {
    Router::new().nest("/{id}", id_routes())
}
