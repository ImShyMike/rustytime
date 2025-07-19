mod db;
mod handlers;
mod models;
mod routes;
mod schema;
mod state;
mod utils;

use axum::http::{Request, Response};
use axum::{Router, routing::get};
use std::net::SocketAddr;
use tower_cookies::CookieManagerLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use db::create_pool;
use routes::create_api_router;
use state::AppState;

#[tokio::main]
async fn main() {
    // load environment variables from .env file
    dotenvy::dotenv().ok();

    // logging stuff
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Starting the rustytime server...");

    // create database connection pool
    let pool = create_pool();
    info!("✅ Database connection pool created");

    // create GitHub OAuth client
    let github_client = handlers::github::create_github_client();
    info!("✅ GitHub OAuth client created");

    // create application state
    let app_state = AppState::new(pool, github_client);

    // main app router
    let app = Router::new()
        .route("/", get(|| async { 
            axum::response::Html(r#"
                <html>
                    <body>
                        <h1>RustyTime OAuth2 Demo</h1>
                        <p>This is a demo application showing GitHub OAuth2 integration with Rust and Axum.</p>
                        <a href="/api/v1/auth/github/login">Login with GitHub</a>
                        <br><br>
                        <a href="/dashboard">Go to Dashboard</a> (requires authentication)
                    </body>
                </html>
            "#)
        }))
        .route("/dashboard", get(handlers::github::dashboard))
        .nest("/api/v1", create_api_router())
        .with_state(app_state)
        .layer(CookieManagerLayer::new())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<axum::body::Body>| {
                    tracing::info_span!(
                        "http",
                        method = %request.method(),
                        uri = %request.uri().path(),
                    )
                })
                .on_response(
                    |response: &Response<axum::body::Body>,
                     latency: std::time::Duration,
                     _span: &tracing::Span| {
                        info!(
                            status = %response.status(),
                            latency = ?latency
                        );
                    },
                ),
        );

    // bind to address
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap_or_else(|_| {
            error!("{}", "Failed to bind address!");
            std::process::exit(1);
        });

    info!("✅ Server running on http://localhost:3000");

    // run the server
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    })
    .await
    .unwrap();
}
