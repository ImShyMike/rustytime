use crate::handlers::admin::{change_user_admin_level, impersonate_user};
use crate::state::AppState;
use crate::utils::middleware;
use axum::{Router, middleware as axum_middleware, routing::get};

/// Route: `/admin`
pub fn admin_routes(app_state: AppState) -> Router<AppState> {
    Router::new()
        .route("/impersonate/{user_id}", get(impersonate_user))
        .route("/admin_level/{user_id}/{admin_level}", get(change_user_admin_level))
        .layer(axum_middleware::from_fn_with_state(
            app_state,
            middleware::require_admin,
        ))
}
