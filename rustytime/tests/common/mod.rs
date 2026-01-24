#![allow(dead_code)]

use axum::Router;
use axum::extract::connect_info::IntoMakeServiceWithConnectInfo;
use axum_prometheus::metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use axum_test::{TestServer, TestServerConfig};
use diesel::prelude::*;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl, basic::BasicClient};
use rustytime_server::{
    db::connection::DbPool,
    models::user::{NewUser, User},
    routes::create_app_router,
    state::AppState,
    utils::cache::AppCache,
    utils::metrics::MetricsTracker,
};
use std::sync::{Arc, OnceLock};
use tokio::sync::RwLock;
use tower_cookies::CookieManagerLayer;

static METRICS_HANDLE: OnceLock<PrometheusHandle> = OnceLock::new();

fn get_or_init_metrics_handle() -> PrometheusHandle {
    METRICS_HANDLE
        .get_or_init(|| {
            PrometheusBuilder::new()
                .install_recorder()
                .expect("Failed to install metrics recorder")
        })
        .clone()
}

/// Test configuration
pub struct TestConfig {
    pub database_url: Option<String>,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            database_url: std::env::var("TEST_DATABASE_URL").ok(),
        }
    }
}

impl TestConfig {
    pub fn has_database(&self) -> bool {
        self.database_url.is_some()
    }

    pub fn database_url(&self) -> String {
        self.database_url
            .clone()
            .expect("TEST_DATABASE_URL must be set for integration tests")
    }
}

/// Fail test if database is not available
#[macro_export]
macro_rules! fail_without_db {
    ($config:expr) => {
        assert!($config.has_database(), "TEST_DATABASE_URL not set");
    };
}

/// Invalid test API key
pub const INVALID_API_KEY: &str = "invalid-api-key";

/// Create a mock heartbeat request payload
pub fn mock_heartbeat_payload() -> serde_json::Value {
    serde_json::json!({
        "entity": "/path/to/file.rs",
        "type": "file",
        "time": 1700000000.123,
        "project": "test-project",
        "language": "Rust",
        "branch": "main"
    })
}

/// Create bulk heartbeat payload
pub fn mock_bulk_heartbeat_payload(count: usize) -> serde_json::Value {
    let heartbeats: Vec<serde_json::Value> = (0..count)
        .map(|i| {
            serde_json::json!({
                "entity": format!("/path/to/file{}.rs", i),
                "type": "file",
                "time": 1700000000.0 + (i as f64 * 60.0),
                "project": "test-project",
                "language": "Rust"
            })
        })
        .collect();
    serde_json::json!(heartbeats)
}

/// Create a database connection pool for tests
pub fn create_test_pool(database_url: &str) -> DbPool {
    use diesel::pg::PgConnection;
    use diesel::r2d2::{self, ConnectionManager};

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .max_size(5)
        .min_idle(Some(1))
        .connection_timeout(std::time::Duration::from_secs(5))
        .build(manager)
        .expect("Failed to create test database pool")
}

/// Create a mock GitHub OAuth client for testing
pub fn create_mock_github_client() -> BasicClient<
    oauth2::EndpointSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointNotSet,
    oauth2::EndpointSet,
> {
    let client_id = ClientId::new("test-client-id".to_string());
    let client_secret = ClientSecret::new("test-client-secret".to_string());
    let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
        .expect("Invalid token endpoint URL");
    let redirect_url = RedirectUrl::new("http://localhost:3000/auth/github/callback".to_string())
        .expect("Invalid redirect URL");

    BasicClient::new(client_id)
        .set_client_secret(client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(redirect_url)
}

/// Test server wrapper
pub struct TestApp {
    pub server: TestServer,
    pub db_pool: DbPool,
}

impl TestApp {
    /// Create a new test application with a real database and full router
    pub async fn new() -> Self {
        let config = TestConfig::default();
        let database_url = config.database_url();
        let db_pool = create_test_pool(&database_url);

        let app = create_test_router_full(db_pool.clone());

        let mut server_config = TestServerConfig::new();
        server_config.save_cookies = true;
        let server = TestServer::new_with_config(app, server_config)
            .expect("Failed to create test server");

        Self { server, db_pool }
    }

    /// Test app if database is available
    pub async fn try_new() -> Option<Self> {
        let config = TestConfig::default();
        if !config.has_database() {
            return None;
        }
        Some(Self::new().await)
    }

    /// Create a test user and return the user with their API key
    pub fn create_test_user(&self, name: &str) -> User {
        use rustytime_server::schema::users;
        use std::time::{SystemTime, UNIX_EPOCH};

        let mut conn = self.db_pool.get().expect("Failed to get DB connection");

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as i64;
        let github_id = 999_000_000 + (timestamp % 1_000_000_000);

        let new_user = NewUser {
            github_id,
            name: name.to_string(),
            avatar_url: format!("https://example.com/avatar/{}", github_id),
            admin_level: 0,
            is_banned: false,
            timezone: "UTC".to_string(),
        };

        diesel::insert_into(users::table)
            .values(&new_user)
            .get_result(&mut conn)
            .expect("Failed to create test user")
    }

    /// Delete a test user and all their data
    pub fn cleanup_test_user(&self, user_id: i32) {
        use rustytime_server::schema::{heartbeats, projects, users};

        let mut conn = self.db_pool.get().expect("Failed to get DB connection");

        diesel::delete(heartbeats::table.filter(heartbeats::user_id.eq(user_id)))
            .execute(&mut conn)
            .ok();

        diesel::delete(projects::table.filter(projects::user_id.eq(user_id)))
            .execute(&mut conn)
            .ok();

        diesel::delete(users::table.filter(users::id.eq(user_id)))
            .execute(&mut conn)
            .ok();
    }
}

/// Create a full test router
fn create_test_router_full(
    db_pool: DbPool,
) -> IntoMakeServiceWithConnectInfo<Router, std::net::SocketAddr> {
    use axum::Extension;

    let github_client = create_mock_github_client();

    let app_state = AppState {
        db_pool,
        github_client,
        http_client: reqwest::Client::new(),
        metrics: MetricsTracker::new(),
        import_store: Arc::new(RwLock::new(None)),
        cache: AppCache::new(),
    };

    let metrics_handle = get_or_init_metrics_handle();

    let api_router = create_app_router(app_state, false, metrics_handle);

    let mut openapi = rustytime_server::docs::get_openapi_docs();
    let mut app: Router = api_router.finish_api(&mut openapi);
    let openapi = Arc::new(openapi);
    app = app.layer(Extension(openapi));
    app = app.layer(CookieManagerLayer::new());

    app.into_make_service_with_connect_info::<std::net::SocketAddr>()
}
