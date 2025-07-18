use axum::{
    Router,
    routing::{get, post},
};
use crate::db::DbPool;

use crate::handlers::user::{create_heartbeats, get_statusbar_today};

pub fn heartbeat_routes() -> Router<DbPool> {
    Router::new()
        // WakaTime compatibility routes
        .route("/heartbeats", post(create_heartbeats))
        .route("/heartbeats.bulk", post(create_heartbeats))
        .route("/statusbar/today", get(get_statusbar_today))
}

pub fn user_routes() -> Router<DbPool> {
    Router::new().nest("/{id}", heartbeat_routes())
}
