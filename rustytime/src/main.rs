#![forbid(unsafe_code)]

mod db;
mod docs;
mod handlers;
mod jobs;
mod models;
mod routes;
mod schema;
mod state;
mod utils;

use axum::{Extension, body::Body, http::Request};
use reqwest::StatusCode;
use std::{env, net::SocketAddr, sync::Arc, time::Duration};
use tower_cookies::CookieManagerLayer;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::{
    compression::CompressionLayer, decompression::DecompressionLayer, limit::RequestBodyLimitLayer,
    normalize_path::NormalizePathLayer, timeout::TimeoutLayer, trace::TraceLayer,
};

use tracing::{error, info};
use tracing_subscriber::layer::SubscriberExt;

use db::connection::create_pool;
use state::AppState;
use utils::http::{CloudflareAwareKeyExtractor, extract_client_ip};
use utils::middleware::cors_layer;

use crate::{
    routes::create_app_router,
    utils::{
        env::{is_production_env, use_cloudflare_headers},
        middleware::cors_allow_all_layer,
    },
};

// about 4 requests per second with a max burst of 60
const DEFAULT_BURST_SIZE: u32 = 60;
const DEFAULT_RATE_LIMIT_REPLENISH_DURATION: Duration = Duration::from_millis(250);
const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(300);

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // load environment variables from .env file
    dotenvy::dotenv().ok();

    // get git sha from environment variable set at build time
    let git_sha = option_env!("GIT_SHA").unwrap_or("unknown");

    // check if running in production
    let is_production = is_production_env();

    // get service name for tracing and profiling
    let service_name =
        env::var("OTEL_SERVICE_NAME").unwrap_or_else(|_| "rustytime-backend".to_string());

    let otel_logging = utils::tracing::init_otlp_logging_layer()?;
    let (otel_log_layer, otel_log_guard) = match otel_logging {
        Some(logging) => {
            let (layer, guard) = logging.into_parts();
            (Some(layer), Some(guard))
        }
        None => (None, None),
    };

    // initialize tracing with OpenTelemetry
    let tracing_config = if cfg!(debug_assertions) {
        init_tracing_opentelemetry::TracingConfig::development()
    } else {
        init_tracing_opentelemetry::TracingConfig::minimal().with_otel(true)
    };

    let _guard = if let Some(layer) = otel_log_layer {
        tracing_config.init_subscriber_ext(|subscriber| subscriber.with(layer))?
    } else {
        tracing_config.init_subscriber()?
    };

    let _otel_log_guard = otel_log_guard;

    // logging stuff
    // tracing_subscriber::fmt()
    //     .with_env_filter(
    //         EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
    //     )
    //     .init();

    // check if should use cloudflare headers
    let use_cloudflare = use_cloudflare_headers();

    let version = env!("CARGO_PKG_VERSION");
    info!(version = %version, git_sha = %git_sha, "🚀 Starting the rustytime server...");

    if use_cloudflare {
        info!("✅ Cloudflare IP extraction enabled!");
    }

    let pyroscope_agent =
        utils::tracing::init_pyroscope_agent(&service_name, git_sha, is_production);

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
        let result = db::seed::seed_database(&pool).await;
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
    let app_state = AppState::new(pool.clone(), github_client);

    // get database URL for sqlx pool
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        let user = env::var("POSTGRES_USER").unwrap_or_else(|_| "username".into());
        let password = env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "password".into());
        let host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".into());
        let db = env::var("POSTGRES_DB").unwrap_or_else(|_| "rustytime".into());
        format!("postgres://{user}:{password}@{host}/{db}")
    });

    // set up metrics recorder for apalis jobs (must be done before setup_jobs)
    let jobs_metrics_handle = jobs::install_metrics_recorder();

    // set up jobs system with apalis
    let sqlx_pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database for jobs");
    let (leaderboard_worker, import_worker, sessions_worker, import_store) =
        jobs::setup_jobs(sqlx_pool, pool.clone()).await;
    app_state.set_import_store(import_store);
    tokio::spawn(leaderboard_worker);
    tokio::spawn(import_worker);
    tokio::spawn(sessions_worker);
    info!("✅ Jobs system started");

    let rate_period = if is_production {
        DEFAULT_RATE_LIMIT_REPLENISH_DURATION
    } else {
        Duration::from_secs(1)
    };
    let burst_size = if is_production {
        DEFAULT_BURST_SIZE
    } else {
        10_000_000
    };

    let governor_conf = GovernorConfigBuilder::default()
        .period(rate_period)
        .burst_size(burst_size)
        .key_extractor(CloudflareAwareKeyExtractor::new(use_cloudflare))
        .use_headers()
        .finish()
        .unwrap();

    let governor_limiter = governor_conf.limiter().clone();

    let interval = Duration::from_secs(60);
    // use a separate background task to clean up
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(interval);
            governor_limiter.retain_recent();
        }
    });

    aide::generate::on_error(|error| {
        println!("{error}");
    });

    // create the main application router
    let api_router = create_app_router(app_state, use_cloudflare, jobs_metrics_handle);
    let mut openapi = docs::get_openapi_docs();
    let mut app: axum::Router = api_router.finish_api(&mut openapi);
    let openapi = Arc::new(openapi);
    app = app.layer(Extension(openapi));
    app = app.layer(CookieManagerLayer::new());

    if is_production {
        // only construct and add the CORS layer when running in production
        app = app.layer(cors_layer());
    } else {
        app = app.layer(cors_allow_all_layer());
    }

    let app = app
        .layer(CompressionLayer::new().gzip(true)) // enable gzip
        .layer(DecompressionLayer::new().gzip(true)) // accept gzip
        .layer(RequestBodyLimitLayer::new(16 * 1024 * 1024)) // 16 MB size limit
        .layer(TimeoutLayer::with_status_code(
            StatusCode::REQUEST_TIMEOUT,
            DEFAULT_REQUEST_TIMEOUT,
        )) // allow long-running operations (e.g., Hackatime imports)
        .layer(GovernorLayer::new(governor_conf)) // rate limiting
        .layer(NormalizePathLayer::trim_trailing_slash()) // normalize paths
        .layer(
            // add request logging
            TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
                let client_ip = extract_client_ip(request);
                tracing::info_span!(
                    "request",
                    method = ?request.method(),
                    uri = %request.uri(),
                    version = ?request.version(),
                    client_ip = %client_ip
                )
            }),
        );

    // bind to address
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);
    let listener = tokio::net::TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port)))
        .await
        .unwrap_or_else(|err| {
            error!("❌ Failed to bind address: {}", err);
            std::process::exit(1);
        });

    info!("✅ Server running on http://localhost:{}", port);

    // run the server
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();

    if let Some(agent) = pyroscope_agent {
        match agent.stop() {
            Ok(ready) => ready.shutdown(),
            Err(err) => error!("❌ Failed to stop Pyroscope profiler: {}", err),
        }
    };

    info!("✅ Server has shut down gracefully.");

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("❌ Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("❌ Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("🔄 Shutdown signal received, terminating...");
}
