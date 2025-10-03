pub mod admin;
pub mod api;
pub mod github;

use crate::handlers;
use crate::handlers::homepage::home_page;
use crate::state::AppState;
use crate::utils::middleware;
use axum::{
    Router, http::StatusCode, middleware as axum_middleware, response::IntoResponse, routing::get,
};

/// Create the main application router
pub fn create_app_router(app_state: AppState) -> Router {
    Router::new()
        // public routes
        .merge(public_routes())
        // auth routes
        .nest("/auth/github", github::github_routes())
        // required authentication
        .merge(protected_routes(app_state.clone()))
        // optional authentication
        // .merge(semi_protected_routes(app_state.clone()))
        // admin routes
        .merge(create_admin_routes(app_state.clone()))
        // API routes
        .nest("/api/v1", api::create_api_router())
        // catch-all fallback for unmatched routes (must be last)
        .fallback(not_found)
        // inject application state
        .with_state(app_state.clone())
        // add metrics tracking middleware
        .layer(axum_middleware::from_fn_with_state(
            app_state,
            middleware::track_metrics,
        ))
}

/// Handler for unmatched routes
async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not Found")
}

/// Public routes that don't require authentication
fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(home_page))
}

/// Protected routes that require authentication
fn protected_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/page/dashboard", get(handlers::dashboard::dashboard))
        .layer(axum_middleware::from_fn_with_state(
            app_state,
            middleware::require_auth,
        ))
}

/// Routes that work with and without authentication
// fn semi_protected_routes(app_state: AppState) -> Router<AppState> {
//     Router::new()
//         .layer(axum_middleware::from_fn_with_state(
//             app_state,
//             middleware::optional_auth,
//         ))
// }

/// Admin routes that require admin privileges
pub fn create_admin_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/page/admin", admin::admin_routes(app_state.clone()))
        .layer(axum_middleware::from_fn_with_state(
            app_state,
            middleware::require_admin,
        ))
}
