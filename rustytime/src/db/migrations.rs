use crate::db::connection::DbPool;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use tracing::{error, info};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/");

/// Run all pending migrations
pub fn run_migrations(
    pool: &DbPool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
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
            Err(e)
        }
    }
}
