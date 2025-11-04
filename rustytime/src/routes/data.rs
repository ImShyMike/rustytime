use crate::handlers::data::project_aliases::{
    add_project_alias, delete_project_alias, project_aliases,
};
use crate::handlers::data::projects::{projects_list, set_project_repo};
use crate::state::AppState;
use axum::{
    Router,
    routing::{delete, get, post, put},
};
/// Route: `/data`
pub fn data_routes() -> Router<AppState> {
    Router::new()
        .route("/projects", get(projects_list))
        .route("/project_aliases/{id}/{alias_id}", put(add_project_alias))
        .route("/project_aliases/{id}", delete(delete_project_alias))
        .route("/project_aliases", get(project_aliases))
        .route("/projects/{id}/repo", post(set_project_repo))
}
