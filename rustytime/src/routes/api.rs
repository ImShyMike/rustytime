use crate::state::AppState;
use crate::handlers::user::{create_heartbeats, get_statusbar_today};
use axum::{
    Router,
    routing::{get, post},
    response::Redirect,
};

pub fn create_api_router() -> Router<AppState> {
    Router::new()
        .route("/", get(|| async { Redirect::permanent("/") }))
        .nest("/users", user_routes())
}

pub fn user_routes() -> Router<AppState> {
    Router::new().nest("/{id}", user_id_routes())
}

fn user_id_routes() -> Router<AppState> {
    Router::new()
        // WakaTime compatibility routes
        .route("/heartbeats", post(create_heartbeats))
        .route("/heartbeats.bulk", post(create_heartbeats))
        .route("/statusbar/today", get(get_statusbar_today))
}
