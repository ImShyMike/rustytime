use std::sync::Arc;

use aide::{
    axum::{routing::get_with, routing::post_with, ApiRouter, IntoApiResponse},
    openapi::OpenApi,
    scalar::Scalar,
};
use axum::{
    routing::{delete as axum_delete, get as axum_get, post as axum_post, put as axum_put},
    Extension, Json, Router,
};
use axum::{http::StatusCode, middleware as axum_middleware};
use axum_prometheus::PrometheusMetricLayer;
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

/// Create the main application router
pub fn create_app_router(app_state: AppState) -> ApiRouter {
    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();
    ApiRouter::new()
        .api_route_with(
            "/docs",
            get_with(
                Scalar::new("/docs/private/api.json")
                    .with_title("rustytime")
                    .axum_handler(),
                |op| op.description("The documentation page.").tag("Documentation"),
            ),
            |p| p.security_requirement("ApiKey"),
        )
        .route("/docs/private/api.json", axum_get(openapi_docs))
        // public routes
        .route("/", axum_get(home_page))
        // auth routes
        .merge(
            Router::new().nest(
                "/auth/github",
                Router::new()
                    .route("/login", axum_get(login))
                    .route("/callback", axum_get(callback))
                    .route("/logout", axum_get(logout))
                    .route("/verify", axum_get(verify_session)),
            ),
        )
        // required authentication
        .merge(
            Router::new()
                .nest(
                    "/page",
                    Router::new()
                        .route("/dashboard", axum_get(dashboard))
                        .route("/projects", axum_get(projects_dashboard))
                        .route("/settings", axum_get(settings_page))
                        .route("/leaderboard", axum_get(leaderboard_page)),
                )
                .nest(
                    "/data",
                    Router::new()
                        .route("/projects", axum_get(projects_list))
                        .route(
                            "/project_aliases/{id}/{alias_id}",
                            axum_put(add_project_alias),
                        )
                        .route("/project_aliases/{id}", axum_delete(delete_project_alias))
                        .route("/project_aliases", axum_get(project_aliases))
                        .route("/projects/{id}/repo", axum_post(set_project_repo)),
                )
                .layer(axum_middleware::from_fn_with_state(
                    app_state.clone(),
                    middleware::require_auth,
                )),
        )
        // admin routes
        .merge(
            ApiRouter::new()
                .route("/page/admin", axum_get(admin_dashboard))
                .nest(
                    "/admin",
                    ApiRouter::new()
                        .route("/impersonate/{user_id}", axum_get(impersonate_user))
                        .route(
                            "/admin_level/{user_id}/{admin_level}",
                            axum_put(change_user_admin_level),
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
            ApiRouter::new()
                .api_route(
                    "/",
                    get_with(
                        home_page,
                        |op| {
                            op.id("api_root_redirect")
                                .summary("Redirect API entry point")
                                .description(
                                    "Redirects to the frontend.",
                                )
                                .tag("Root")
                        },
                    ),
                )
                .nest(
                    "/users",
                    ApiRouter::new().nest(
                        "/{id}",
                        ApiRouter::new()
                            // WakaTime compatibility routes
                            .api_route(
                                "/heartbeats",
                                post_with(create_heartbeats, |op| {
                                    op.id("create_heartbeat")
                                        .summary("Create a heartbeat")
                                        .description(
                                            "Accepts a single heartbeat payload in the same format used by the WakaTime client.",
                                        )
                                        .tag("Heartbeats")
                                }),
                            )
                            .api_route(
                                "/heartbeats.bulk",
                                post_with(create_heartbeats, |op| {
                                    op.id("create_heartbeats_bulk")
                                        .summary("Create multiple heartbeats")
                                        .description(
                                            "Bulk ingestion endpoint compatible with WakaTime's heartbeats.bulk route.",
                                        )
                                        .tag("Heartbeats")
                                }),
                            )
                            .api_route(
                                "/statusbar/today",
                                get_with(get_statusbar_today, |op| {
                                    op.id("statusbar_today")
                                        .summary("Status bar stats for today")
                                        .description(
                                            "Returns the coding data for the current day.",
                                        )
                                        .tag("Users")
                                }),
                            ),
                    ),
                ),
        )
        // metrics endpoint
        .route(
            "/metrics",
            axum_get(|| async move { metric_handle.render() }),
        )
        // method not allowed fallback
        // .method_not_allowed_fallback(method_not_allowed) // aide does not support this :(
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
async fn not_found() -> impl IntoApiResponse {
    (StatusCode::NOT_FOUND, "Not Found")
}

/// Serve the generated OpenAPI document
async fn openapi_docs(
    Extension(openapi): Extension<Arc<OpenApi>>,
) -> impl IntoApiResponse {
    Json(openapi.as_ref().clone())
}

// Handler for method not allowed responses
// async fn method_not_allowed() -> impl IntoApiResponse {
//     (StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed")
// }

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{self, Body},
        http,
        response::IntoResponse,
    };
    use diesel::{
        pg::PgConnection,
        r2d2::{ConnectionManager, Pool},
    };
    use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
    use tower::ServiceExt;

    fn build_test_state() -> AppState {
        let manager = ConnectionManager::<PgConnection>::new("postgres://invalid");
        let db_pool = Pool::builder()
            .max_size(1)
            .min_idle(Some(0))
            .build_unchecked(manager);

        let github_client = BasicClient::new(ClientId::new("client-id".into()))
            .set_client_secret(ClientSecret::new("client-secret".into()))
            .set_auth_uri(AuthUrl::new("https://example.test/auth".into()).unwrap())
            .set_token_uri(TokenUrl::new("https://example.test/token".into()).unwrap())
            .set_redirect_uri(RedirectUrl::new("https://example.test/callback".into()).unwrap());

        AppState {
            db_pool,
            github_client,
            http_client: reqwest::Client::new(),
            metrics: MetricsTracker::new(),
        }
    }

    use crate::utils::metrics::MetricsTracker;
    use oauth2::basic::BasicClient;

    #[tokio::test]
    async fn create_app_router_wires_routes_and_state() {
        let app_state = build_test_state();
        let app = create_app_router(app_state);

        let response = app
            .clone()
            .oneshot(
                http::Request::builder()
                    .uri("/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("router should produce a response");
        assert!(response.status().is_redirection());
        assert_eq!(
            response.headers().get(http::header::LOCATION).unwrap(),
            "http://localhost:5173"
        );

        let metrics_response = app
            .oneshot(
                http::Request::builder()
                    .uri("/metrics")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .expect("metrics endpoint should respond");
        assert_eq!(metrics_response.status(), StatusCode::OK);
        let body_bytes = body::to_bytes(metrics_response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert!(!body_bytes.is_empty());
    }

    #[tokio::test]
    async fn not_found_returns_404_payload() {
        let response = not_found().await.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body_bytes = body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        assert_eq!(body_bytes.as_ref(), b"Not Found");
    }
}
