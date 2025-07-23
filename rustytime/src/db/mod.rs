use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenvy::dotenv;
use std::env;
use tracing::{error, info};

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

/// Run all pending migrations
pub fn run_migrations(pool: &DbPool) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let mut conn = pool.get()?;
    info!("🔄 Running database migrations...");
    
    match conn.run_pending_migrations(MIGRATIONS) {
        Ok(migrations_run) => {
            if migrations_run.is_empty() {
                info!("✅ No pending migrations to run");
            } else {
                info!("✅ Successfully ran {} migrations", migrations_run.len());
                for migration in migrations_run {
                    info!("  - Applied migration: {}", migration);
                }
            }
            Ok(())
        }
        Err(e) => {
            error!("❌ Failed to run migrations: {}", e);
            Err(e.into())
        }
    }
}

/// Create a new database connection pool
pub fn create_pool() -> DbPool {
    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://localhost/rustytime".to_string());

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}
