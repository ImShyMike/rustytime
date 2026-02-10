use std::sync::Arc;

use crate::handlers::admin::change_user_admin_level;
use crate::handlers::api::user::{create_heartbeats, get_statusbar_today};
use crate::handlers::data::import::{import_heartbeats, import_status};
use crate::handlers::data::project_aliases::{
    add_project_alias, delete_project_alias, project_aliases,
};
use crate::handlers::data::projects::{projects_list, update_project};
use crate::handlers::github::{callback, login, logout, verify_session};
use crate::handlers::homepage::home_page;
use crate::handlers::info::info;
use crate::handlers::page::admin::admin_dashboard;
use crate::handlers::page::admin::impersonate_user;
use crate::handlers::page::dashboard::dashboard;
use crate::handlers::page::imports::admin_imports;
use crate::handlers::page::leaderboard::leaderboard_page;
use crate::handlers::page::projects::projects_dashboard;
use crate::handlers::page::settings::{settings_page, update_settings};
use crate::state::AppState;
use crate::utils::middleware;
use aide::axum::routing::{delete_with, get_with, post_with, put_with};
use aide::{
    axum::{ApiRouter, IntoApiResponse},
    openapi::OpenApi,
    scalar::Scalar,
};
use axum::{Extension, Json, routing::get as axum_get};
use axum::{http::StatusCode, middleware as axum_middleware};
use axum_prometheus::PrometheusMetricLayerBuilder;
use axum_prometheus::metrics_exporter_prometheus::PrometheusHandle;
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};

/// Create the main application router
pub fn create_app_router(
    app_state: AppState,
    use_cloudflare: bool,
    metrics_handle: PrometheusHandle,
) -> ApiRouter {
    let prometheus_layer = PrometheusMetricLayerBuilder::new()
        .with_ignore_patterns(&["/metrics", "/health"])
        .build();
    ApiRouter::new()
        .api_route(
            "/docs",
            get_with(
                Scalar::new("/docs/private/api.json")
                    .with_title("rustytime")
                    .axum_handler(),
                |op| op.hidden(true)
            )
        )
        // serve OpenAPI docs
        .route("/docs/private/api.json", axum_get(openapi_docs))
        // public routes
        .route("/", axum_get(home_page))
        .nest(
            "/page",
            ApiRouter::new()
                .api_route("/leaderboard", get_with(leaderboard_page, |op| {
                    op.id("leaderboard_page")
                        .summary("Leaderboard Page")
                        .description("Data for the leaderboard page.")
                        .tag("Pages")
                })),
        )
        // auth routes
        .merge(
            ApiRouter::new().nest(
                "/auth/github",
                ApiRouter::new()
                    .api_route("/login", get_with(login, |op| {
                        op.id("github_login")
                            .summary("GitHub OAuth Login")
                            .description("Initiates the GitHub OAuth login process.")
                            .tag("Authentication")
                    }))
                    .api_route("/callback", get_with(callback, |op| {
                        op.id("github_callback")
                            .summary("GitHub OAuth Callback")
                            .description("Handles the callback from GitHub after OAuth authentication.")
                            .tag("Authentication")
                    }))
                    .api_route("/logout", get_with(logout, |op| {
                        op.id("github_logout")
                            .summary("Logout User")
                            .description("Logs out the currently authenticated user.")
                            .tag("Authentication")
                            .security_requirement("Authenticated")
                    }))
                    .api_route("/verify", get_with(verify_session, |op| {
                        op.id("verify_session")
                            .summary("Verify User Session")
                            .description("Verifies the current user's session and returns user information.")
                            .tag("Authentication")
                    })),
            ),
        )
        // required authentication
        .merge(
            ApiRouter::new()
                .nest(
                    "/page",
                    ApiRouter::new()
                        .api_route("/dashboard", get_with(dashboard, |op| {
                            op.id("user_dashboard")
                                .summary("User Dashboard Page")
                                .description("Data for the dashboard page.")
                                .tag("Pages")
                                .security_requirement("Authenticated")
                        }))
                        .api_route("/projects", get_with(projects_dashboard, |op| {
                            op.id("projects_dashboard")
                                .summary("Projects Dashboard Page")
                                .description("Data for the projects page.")
                                .tag("Pages")
                                .security_requirement("Authenticated")
                        }))
                        .api_route("/settings", get_with(settings_page, |op| {
                            op.id("settings_page")
                                .summary("User Settings Page")
                                .description("Data for the settings page.")
                                .tag("Pages")
                                .security_requirement("Authenticated")
                        })),
                )
                .nest(
                    "/data",
                    ApiRouter::new()
                        .api_route("/projects", get_with(projects_list, |op| {
                            op.id("projects_list")
                                .summary("Projects List")
                                .description("Retrieves the list of projects.")
                                .tag("Data")
                                .security_requirement("Authenticated")
                        }))
                        .api_route(
                            "/project_aliases/{id}/{alias_id}",
                            put_with(add_project_alias, |op| {
                                op.id("add_project_alias")
                                    .summary("Add Project Alias")
                                    .description(
                                        "Adds an alias to the specified project.",
                                    )
                                    .tag("Data")
                                    .security_requirement("Authenticated")
                            },
                        ))
                        .api_route("/project_aliases/{id}", delete_with(delete_project_alias, |op| {
                            op.id("delete_project_alias")
                                .summary("Delete Project Alias")
                                .description(
                                    "Deletes an alias from the specified project.",
                                )
                                .tag("Data")
                                .security_requirement("Authenticated")
                        }))
                        .api_route("/project_aliases", get_with(project_aliases, |op| {
                            op.id("list_project_aliases")
                                .summary("List Project Aliases")
                                .description(
                                    "Retrieves all project aliases.",
                                )
                                .tag("Data")
                                .security_requirement("Authenticated")
                        }))
                        .api_route("/projects/{id}", put_with(update_project, |op| {
                            op.id("update_project")
                                .summary("Update Project")
                                .description(
                                    "Updates the specified project's settings (repo_url, hidden).",
                                )
                                .tag("Data")
                                .security_requirement("Authenticated")
                        }))
                        .api_route("/settings", put_with(update_settings, |op| {
                            op.id("update_settings")
                                .summary("Update User Settings")
                                .description("Updates the authenticated user's settings.")
                                .tag("Data")
                                .security_requirement("Authenticated")
                        }))
                        .api_route(
                            "/import",
                            post_with(import_heartbeats, |op| {
                                op.id("import_heartbeats")
                                    .summary("Start Import Job")
                                    .description(
                                        "Starts a background job to import heartbeats from Hackatime using the provided api key.",
                                    )
                                    .tag("Data")
                            }),
                        )
                        .api_route(
                            "/import/status",
                            get_with(import_status, |op| {
                                op.id("import_status")
                                    .summary("Get Import Job Status")
                                    .description(
                                        "Gets the status of the latest import job for the user.",
                                    )
                                    .tag("Data")
                            }),
                        )
                )
                .layer(axum_middleware::from_fn_with_state(
                    app_state.clone(),
                    middleware::require_auth,
                )),
        )
        // admin routes
        .merge(
            ApiRouter::new()
                .api_route("/page/admin", get_with(admin_dashboard, |op| {
                    op.id("admin_dashboard")
                        .summary("Admin Dashboard Page")
                        .description("Data for the admin page.")
                        .tag("Pages")
                        .security_requirement("Authenticated")
                }))
                .nest(
                    "/admin",
                    ApiRouter::new()
                        .api_route("/impersonate/{user_id}", get_with(impersonate_user, |op| {
                            op.id("impersonate_user")
                                .summary("Impersonate User")
                                .description(
                                    "Allows an admin to impersonate another user.",
                                )
                                .tag("Admin")
                                .security_requirement("Authenticated")
                        }))
                        .api_route(
                            "/admin_level/{user_id}/{admin_level}",
                             put_with(change_user_admin_level, |op| {
                                op.id("change_user_admin_level")
                                    .summary("Change User Admin Level")
                                    .description(
                                        "Allows an admin to change another user's admin level.",
                                    )
                                    .tag("Admin")
                                    .security_requirement("Authenticated")
                            }
                        ))
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
        // owner routes
        .merge(
            ApiRouter::new()
                .api_route("/page/imports", get_with(admin_imports, |op| {
                    op.id("admin_imports")
                        .summary("Admin Imports Page")
                        .description("Data for the admin imports page.")
                        .tag("Pages")
                        .security_requirement("Authenticated")
                }))
                .layer(axum_middleware::from_fn_with_state(
                    app_state.clone(),
                    middleware::require_owner,
                )),
        )
        // API routes
        .nest(
            "/api/v1",
            ApiRouter::new()
                .route(
                    "/",
                    axum_get(home_page),
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
                                        .tag("WakaTime Compatibility")
                                        .security_requirement("ApiKey")
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
                                        .tag("WakaTime Compatibility")
                                        .security_requirement("ApiKey")
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
                                        .tag("WakaTime Compatibility")
                                        .security_requirement("ApiKey")
                                }),
                            ),
                    ),
                ),
        )
        // metrics endpoint
        .route(
            "/metrics",
            axum_get(move || async move { metrics_handle.render() }),
        )
        // health check endpoint
        .api_route("/health", get_with(|| async { "OK" }, |op| {
            op.id("health_check")
                .summary("Health Check Endpoint")
                .description("Returns OK if the server is running.")
                .tag("Health")
        }))
        // info endpoint
        .api_route("/info", get_with(info, |op| {
            op.id("info_endpoint")
                .summary("Server Info Endpoint")
                .description("Returns server name, version, and git SHA.")
                .tag("Info")
        }))
        // include trace context as header into the response
        .layer(OtelInResponseLayer)
        //start OpenTelemetry trace on incoming request
        .layer(OtelAxumLayer::default().try_extract_client_ip(use_cloudflare))
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
async fn openapi_docs(Extension(openapi): Extension<Arc<OpenApi>>) -> impl IntoApiResponse {
    Json(openapi.as_ref().clone())
}

// Handler for method not allowed responses
// async fn method_not_allowed() -> impl IntoApiResponse {
//     (StatusCode::METHOD_NOT_ALLOWED, "Method Not Allowed")
// }
