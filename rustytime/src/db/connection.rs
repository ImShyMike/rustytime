use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use dotenvy::dotenv;
use std::env;
pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
use tracing::{error, info};

/// Create a new database connection pool
pub fn create_pool() -> DbPool {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        let user = env::var("POSTGRES_USER").unwrap_or_else(|_| "username".into());
        let password = env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "password".into());
        let host = env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".into());
        let db = env::var("POSTGRES_DB").unwrap_or_else(|_| "rustytime".into());
        format!("postgres://{user}:{password}@{host}/{db}")
    });

    info!("🔄 Trying to connect to the database...");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .max_size(15)
        .min_idle(Some(5))
        .connection_timeout(std::time::Duration::from_secs(10))
        .idle_timeout(Some(std::time::Duration::from_secs(300)))
        .max_lifetime(Some(std::time::Duration::from_secs(1800)))
        .build(manager)
        .unwrap_or_else(|e| {
            error!("❌ Database connection failed: {}", e);
            std::process::exit(1);
        });
    info!("✅ Database connection pool created");
    pool
}
