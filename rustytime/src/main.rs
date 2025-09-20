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
use std::{net::SocketAddr, time::Duration};
use tower_cookies::CookieManagerLayer;
use tower_http::{
    compression::CompressionLayer, decompression::DecompressionLayer, limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer, trace::TraceLayer,
};

use tracing::{error, info};

use db::connection::create_pool;
use state::AppState;
use utils::logging::init_tracing;
use utils::middleware::cors_layer;

use crate::routes::create_app_router;

#[tokio::main]
async fn main() {
    // load environment variables from .env file
    dotenvy::dotenv().ok();

    // logging stuff
    init_tracing();

    info!("🚀 Starting the rustytime server...");

    // create database connection pool
    let pool = create_pool();

    // run database migrations
    if let Err(e) = db::migrations::run_migrations(&pool) {
        error!("❌ Failed to run migrations: {}", e);
        std::process::exit(1);
    }

    // seed database if enabled
    #[cfg(feature = "seed")]
    {
        let mut conn = pool.get().unwrap_or_else(|e| {
            error!("❌ Failed to get database connection for seeding: {}", e);
            std::process::exit(1);
        });

        let result = db::seed::seed_database(&mut conn);
        match result {
            Ok(_) => info!("✅ Database seeding completed"),
            Err(e) => {
                error!("❌ Database seeding failed: {}", e);
                std::process::exit(1);
            }
        }
    }

    // create GitHub OAuth client
    let github_client = handlers::github::create_github_client();
    info!("✅ GitHub OAuth client created");

    // create application state
    let app_state = AppState::new(pool, github_client);

    // create the main application router
    let app = create_app_router(app_state)
        .layer(CookieManagerLayer::new())
        .layer(cors_layer()) // add CORS
        .layer(CompressionLayer::new().gzip(true)) // enable gzip
        .layer(DecompressionLayer::new().gzip(true)) // accept gzip
        .layer(RequestBodyLimitLayer::new(10 * 1024 * 1024)) // 10 MB size limit
        .layer(TimeoutLayer::new(Duration::from_secs(10))) // 10 seconds timeout
        .layer(
            // add request logging
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
        .unwrap_or_else(|err| {
            error!("❌ Failed to bind address: {}", err);
            std::process::exit(1);
        });

    info!("✅ Server running on http://localhost:3000");

    // run the server
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(async {
        tokio::signal::ctrl_c().await.unwrap_or_else(|err| {
            error!("❌ Failed to install Ctrl+C handler: {}", err);
            std::process::exit(1);
        })
    })
    .await
    .unwrap();
}
