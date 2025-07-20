pub mod api;
pub mod github;
pub mod user;

use crate::handlers;
use crate::handlers::homepage::home_page;
use crate::state::AppState;
use crate::utils::middleware;
use axum::{Router, middleware as axum_middleware, routing::get};

pub fn create_app_router(app_state: AppState) -> Router {
    Router::new()
        // public routes
        // .merge(public_routes())
        // auth routes
        .nest("/auth/github", github::github_routes())
        // required authentication
        .merge(protected_routes(app_state.clone()))
        // optional authentication
        .merge(semi_protected_routes(app_state.clone()))
        // API routes
        .nest("/api/v1", api::create_api_router())
        .with_state(app_state)
}

/// public routes that don't require authentication
// fn public_routes() -> Router<AppState> {

// }

/// protected routes that require authentication
fn protected_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(handlers::dashboard::dashboard))
        .layer(axum_middleware::from_fn_with_state(
            app_state,
            middleware::require_auth,
        ))
}

// routes that work with and without authentication
fn semi_protected_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(home_page))
        .layer(axum_middleware::from_fn_with_state(
            app_state,
            middleware::optional_auth,
        ))
}
