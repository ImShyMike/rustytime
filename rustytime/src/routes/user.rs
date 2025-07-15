use axum::{Router, routing::get};

use crate::handlers::user::{get_user, list_users};

pub fn user_routes() -> Router {
    Router::new()
        .route("/", get(list_users))
        .route("/{id}", get(get_user))
}
