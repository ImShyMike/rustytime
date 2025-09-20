use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use dotenvy::dotenv;
use std::env;
pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;
use tracing::{error, info};

/// Create a new database connection pool
pub fn create_pool() -> DbPool {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://username:password@localhost/rustytime".to_string());

    info!("🔄 Trying to connect to the database...");

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .max_size(15)
        .min_idle(Some(5))
        .build(manager)
        .unwrap_or_else(|e| {
            error!("❌ Database connection failed: {}", e);
            std::process::exit(1);
        });
    info!("✅ Database connection pool created");
    pool
}
