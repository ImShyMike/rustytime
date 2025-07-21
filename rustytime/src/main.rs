mod db;
mod handlers;
mod models;
mod routes;
mod schema;
mod state;
mod utils;

use axum::{
    body::Body,
    http::{Request, Response},
};
use std::net::SocketAddr;
use tower_cookies::CookieManagerLayer;
use tower_http::trace::TraceLayer;
use tracing::{error, info};

use db::create_pool;
use state::AppState;
use utils::logging::init_tracing;

use crate::routes::create_app_router;

#[tokio::main]
async fn main() {
    // load environment variables from .env file
    dotenvy::dotenv().ok();

    // logging stuff
    init_tracing();

    info!("🚀 Starting the rustytime server...");

    // tell the user if running in development mode
    #[cfg(debug_assertions)]
    info!("🔧 Running in development mode! File watching enabled.");

    // create database connection pool
    let pool = create_pool();
    info!("✅ Database connection pool created");

    // create GitHub OAuth client
    let github_client = handlers::github::create_github_client();
    info!("✅ GitHub OAuth client created");

    // create application state
    let app_state = AppState::new(pool, github_client);

    // create the main application router
    let app = create_app_router(app_state)
        .layer(CookieManagerLayer::new())
        .layer(
            // setup request logging
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<Body>| {
                    tracing::info_span!(
                        "http",
                        method = %request.method(),
                        uri = %request.uri().path(),
                    )
                })
                .on_response(
                    |response: &Response<Body>,
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
