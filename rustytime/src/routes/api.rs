use crate::handlers::api::user::{create_heartbeats, get_statusbar_today};
use crate::state::AppState;
use axum::{
    Router,
    response::Redirect,
    routing::{get, post},
};

/// Route: `/api/v1`
pub fn create_api_router() -> Router<AppState> {
    Router::new()
        .route("/", get(|| async { Redirect::permanent("/") }))
        .nest("/users", user_routes())
}

// Route: `/api/v1/admin`
// pub fn create_admin_api_router() -> Router<AppState> {
//     Router::new()
//         .route(
//             "/admin_level/{user_id}/{admin_level}",
//             put(change_user_admin_level),
//         )
// }

/// Route: `/api/v1/users`
pub fn user_routes() -> Router<AppState> {
    Router::new().nest("/{id}", user_id_routes())
}

/// Route: `/api/v1/users/{id}`
fn user_id_routes() -> Router<AppState> {
    Router::new()
        // WakaTime compatibility routes
        .route("/heartbeats", post(create_heartbeats))
        .route("/heartbeats.bulk", post(create_heartbeats))
        .route("/statusbar/today", get(get_statusbar_today))
}
