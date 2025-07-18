pub mod user;

use axum::{Router, routing::get};
use crate::db::DbPool;

pub fn create_api_router() -> Router<DbPool> {
    Router::new()
        .route(
            "/",
            get(|| async { axum::response::Redirect::permanent("/") }),
        )
        .nest("/users", user::user_routes())
}
