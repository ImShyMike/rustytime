use crate::routes::user;
use crate::state::AppState;
use axum::{Router, response::Redirect, routing::get};

pub fn create_api_router() -> Router<AppState> {
    Router::new()
        .route("/", get(|| async { Redirect::permanent("/") }))
        .nest("/users", user::user_routes())
}
