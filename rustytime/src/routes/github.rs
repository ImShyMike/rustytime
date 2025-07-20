use crate::handlers::github::{callback, login, logout};
use crate::state::AppState;
use axum::{Router, routing::get};

pub fn github_routes() -> Router<AppState> {
    Router::new()
        .route("/login", get(login))
        .route("/callback", get(callback))
        .route("/logout", get(logout))
}
