use crate::handlers::github::{callback, login, logout};
use crate::state::AppState;
use axum::{Router, routing::get};

pub fn github_routes() -> Router<AppState> {
    Router::new()
        .route("/github/login", get(login))
        .route("/github/callback", get(callback))
        .route("/github/logout", get(logout))
}
