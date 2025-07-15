use axum::{Router, routing::get};

use crate::handlers::project::{get_project, list_projects};

pub fn project_routes() -> Router {
    Router::new()
        .route("/", get(list_projects))
        .route("/{id}", get(get_project))
}
