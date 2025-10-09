use crate::handlers::admin::{admin_dashboard, impersonate_user};
use crate::state::AppState;
use crate::utils::middleware;
use axum::{Router, middleware as axum_middleware, routing::get};

/// Route: `/admin`
pub fn admin_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(admin_dashboard))
        .route("/impersonate/{user_id}", get(impersonate_user))
        .layer(axum_middleware::from_fn_with_state(
            app_state,
            middleware::require_admin,
        ))
}
