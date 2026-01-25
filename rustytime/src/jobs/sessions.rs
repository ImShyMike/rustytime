use std::str::FromStr;

use apalis::{
    layers::{WorkerBuilderExt, prometheus::PrometheusLayer},
    prelude::{Data, WorkerBuilder},
};
use apalis_cron::{CronStream, Tick};
use cron::Schedule;
use tokio::signal::ctrl_c;

use diesel::Connection;

use crate::db::connection::DbPool;
use crate::models::session::Session;

const SESSION_RETENTION_DAYS: i64 = 30;

fn cleanup_expired_sessions(pool: &DbPool) -> Result<(), diesel::result::Error> {
    let mut conn = pool.get().map_err(|e| {
        tracing::error!(error = ?e, "Failed to get connection for session cleanup");
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(e.to_string()),
        )
    })?;

    conn.transaction(|conn| {
        let deleted = Session::delete_expired(conn, SESSION_RETENTION_DAYS)?;

        tracing::debug!(deleted, "Cleaned up old leaderboard entries");

        Ok(())
    })
}

async fn run_cleanup(_tick: Tick, pool: Data<DbPool>) {
    if let Err(e) = cleanup_expired_sessions(&pool) {
        tracing::error!(error = ?e, "Failed to cleanup old leaderboard entries");
    }
}

pub async fn setup(diesel_pool: DbPool) -> impl std::future::Future<Output = ()> {
    let cleanup_schedule =
        Schedule::from_str("0 0 0 * * *").expect("valid cron: daily at midnight");

    let cleanup_pool = diesel_pool;

    let cleanup_worker = WorkerBuilder::new("sessions-cleanup")
        .backend(CronStream::new(cleanup_schedule))
        .enable_tracing()
        .layer(PrometheusLayer::default())
        .catch_panic()
        .data(cleanup_pool)
        .build(run_cleanup);

    async move {
        tokio::select! {
            _ = cleanup_worker.run() => {}
            _ = ctrl_c() => {
                tracing::info!("Shutting down session workers");
            }
        }
    }
}
