pub mod import;
mod leaderboard;

use apalis_postgres::PostgresStorage;
use axum_prometheus::metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use chrono::{Datelike, NaiveDate};
use sqlx::PgPool;

use crate::db::connection::DbPool;

pub fn install_metrics_recorder() -> PrometheusHandle {
    PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install metrics recorder")
}

pub async fn setup_jobs(
    sqlx_pool: PgPool,
    diesel_pool: DbPool,
) -> (
    impl std::future::Future<Output = ()>,
    impl std::future::Future<Output = ()>,
    import::ImportStore,
) {
    PostgresStorage::setup(&sqlx_pool).await.unwrap();

    let import_store = import::create_storage(&sqlx_pool).await;
    let leaderboard_worker = leaderboard::setup(sqlx_pool.clone(), diesel_pool.clone()).await;
    let import_worker = import::setup(sqlx_pool, diesel_pool).await;

    (leaderboard_worker, import_worker, import_store)
}

#[inline(always)]
pub fn get_week_start(date: NaiveDate) -> NaiveDate {
    let weekday = date.weekday().num_days_from_monday();
    date - chrono::Duration::days(weekday as i64)
}
