pub mod project;
pub mod user;

use axum::{Router, routing::get};

pub fn create_api_router() -> Router {
    Router::new()
        .route("/", get(|| async { "API is up" }))
        .nest("/users", user::user_routes())
        .nest("/projects", project::project_routes())
}
