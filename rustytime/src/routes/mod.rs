pub mod github;
pub mod user;

use crate::state::AppState;
use axum::{Router, routing::get};

pub fn create_api_router() -> Router<AppState> {
    Router::new()
        .route(
            "/",
            get(|| async { axum::response::Redirect::permanent("/") }),
        )
        .nest("/users", user::user_routes())
}
