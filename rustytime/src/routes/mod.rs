pub mod admin;
pub mod api;
pub mod github;

use crate::handlers;
use crate::handlers::homepage::home_page;
use crate::state::AppState;
use crate::utils::middleware;
use axum::{Router, middleware as axum_middleware, routing::get};

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
        .merge(semi_protected_routes(app_state.clone()))
        // admin routes
        .merge(create_admin_routes(app_state.clone()))
        // API routes
        .nest("/api/v1", api::create_api_router())
        // inject application state
        .with_state(app_state.clone())
        // add metrics tracking middleware
        .layer(axum_middleware::from_fn_with_state(
            app_state,
            middleware::track_metrics,
        ))
}

/// Public routes that don't require authentication
fn public_routes() -> Router<AppState> {
    Router::new()
}

/// Protected routes that require authentication
fn protected_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(handlers::dashboard::dashboard))
        .layer(axum_middleware::from_fn_with_state(
            app_state,
            middleware::require_auth,
        ))
}

/// Routes that work with and without authentication
fn semi_protected_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(home_page))
        .layer(axum_middleware::from_fn_with_state(
            app_state,
            middleware::optional_auth,
        ))
}

/// Admin routes that require admin privileges
pub fn create_admin_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/admin", admin::admin_routes(app_state.clone()))
        .layer(axum_middleware::from_fn_with_state(
            app_state,
            middleware::require_admin,
        ))
}
