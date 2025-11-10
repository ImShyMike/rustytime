use crate::handlers::admin::change_user_admin_level;
use crate::handlers::api::user::{create_heartbeats, get_statusbar_today};
use crate::handlers::data::project_aliases::{
    add_project_alias, delete_project_alias, project_aliases,
};
use crate::handlers::data::projects::{projects_list, set_project_repo};
use crate::handlers::github::{callback, login, logout, verify_session};
use crate::handlers::homepage::home_page;
use crate::handlers::page::admin::admin_dashboard;
use crate::handlers::page::admin::impersonate_user;
use crate::handlers::page::dashboard::dashboard;
use crate::handlers::page::leaderboard::leaderboard_page;
use crate::handlers::page::projects::projects_dashboard;
use crate::handlers::page::settings::settings_page;
use crate::state::AppState;
use crate::utils::middleware;
use axum::response::Redirect;
use axum::routing::{delete, get, post, put};
use axum::{Router, http::StatusCode, middleware as axum_middleware, response::IntoResponse};
use axum_prometheus::PrometheusMetricLayer;

/// Create the main application router
pub fn create_app_router(app_state: AppState) -> Router {
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();
    Router::new()
        // public routes
        .merge(Router::new().route("/", get(home_page)))
        // auth routes
        .nest(
            "/auth/github",
            Router::new()
                .route("/login", get(login))
                .route("/callback", get(callback))
                .route("/logout", get(logout))
                .route("/verify", get(verify_session)),
        )
        // required authentication
        .merge(
            Router::new()
                .nest(
                    "/page",
                    Router::new()
                        .route("/dashboard", get(dashboard))
                        .route("/projects", get(projects_dashboard))
                        .route("/settings", get(settings_page))
                        .route("/leaderboard", get(leaderboard_page)),
                )
                .nest(
                    "/data",
                    Router::new()
                        .route("/projects", get(projects_list))
                        .route("/project_aliases/{id}/{alias_id}", put(add_project_alias))
                        .route("/project_aliases/{id}", delete(delete_project_alias))
                        .route("/project_aliases", get(project_aliases))
                        .route("/projects/{id}/repo", post(set_project_repo)),
                )
                .layer(axum_middleware::from_fn_with_state(
                    app_state.clone(),
                    middleware::require_auth,
                )),
        )
        // admin routes
        .merge(
            Router::new()
                .route("/page/admin", get(admin_dashboard))
                .nest(
                    "/admin",
                    Router::new()
                        .route("/impersonate/{user_id}", get(impersonate_user))
                        .route(
                            "/admin_level/{user_id}/{admin_level}",
                            put(change_user_admin_level),
                        )
                        .layer(axum_middleware::from_fn_with_state(
                            app_state.clone(),
                            middleware::require_admin,
                        )),
                )
                .layer(axum_middleware::from_fn_with_state(
                    app_state.clone(),
                    middleware::require_admin,
                )),
        )
        // API routes
        .nest(
            "/api/v1",
            Router::new()
                .route("/", get(|| async { Redirect::permanent("/") }))
                .nest(
                    "/users",
                    Router::new().nest(
                        "/{id}",
                        Router::new()
                            // WakaTime compatibility routes
                            .route("/heartbeats", post(create_heartbeats))
                            .route("/heartbeats.bulk", post(create_heartbeats))
                            .route("/statusbar/today", get(get_statusbar_today)),
                    ),
                ),
        )
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
