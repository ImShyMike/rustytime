mod handlers;
mod routes;

use axum::http::{Request, Response};
use axum::{Router, routing::get};
use std::net::SocketAddr;
use tower_http::trace::TraceLayer;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use routes::create_api_router;

#[tokio::main]
async fn main() {
    // logging stuff
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_max_level(tracing::Level::INFO)
        .init();

    info!("🚀 Starting the rustytime server...");

    // main app router
    let app = Router::new()
        .route("/", get(|| async { "Server is up" }))
        .nest("/api", create_api_router())
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
