pub mod admin;
pub mod api;
pub mod github;

use crate::handlers::homepage::home_page;
use crate::handlers::{self, admin::admin_dashboard};
use crate::state::AppState;
use crate::utils::middleware;
use axum::routing::get;
use axum::{Router, http::StatusCode, middleware as axum_middleware, response::IntoResponse};
use axum_prometheus::PrometheusMetricLayer;

/// Create the main application router
pub fn create_app_router(app_state: AppState) -> Router {
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();
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
        // metrics endpoint
        .route("/metrics", get(|| async move { metric_handle.render() }))
        // catch-all fallback for unmatched routes (must be last)
        .fallback(not_found)
        // inject application state
        .with_state(app_state.clone())
        // add metrics tracking middleware
        .layer(axum_middleware::from_fn_with_state(
            app_state,
            middleware::track_metrics,
        ))
        .layer(prometheus_layer)
}

/// Handler for unmatched routes
async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Not Found")
}

/// Public routes that don't require authentication
fn public_routes() -> Router<AppState> {
    Router::new().route("/", get(home_page))
}

fn protected_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .nest(
            "/page",
            Router::new()
                .route("/dashboard", get(handlers::dashboard::dashboard))
                .route("/projects", get(handlers::projects::projects_dashboard))
                .route("/settings", get(handlers::settings::settings_page)),
        )
        .layer(axum_middleware::from_fn_with_state(
            app_state,
            middleware::require_auth,
        ))
}

// Routes that work with and without authentication
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
        .nest("/page", Router::new().route("/admin", get(admin_dashboard)))
        .nest("/admin", admin::admin_routes(app_state.clone()))
        .layer(axum_middleware::from_fn_with_state(
            app_state,
            middleware::require_admin,
        ))
}
