pub mod import;
mod leaderboard;
mod sessions;

use apalis_postgres::PostgresStorage;
use axum_prometheus::metrics_exporter_prometheus::{PrometheusBuilder, PrometheusHandle};
use sqlx::PgPool;
use std::future::Future;

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
    impl Future<Output = ()>,
    impl Future<Output = ()>,
    impl Future<Output = ()>,
    import::ImportStore,
) {
    PostgresStorage::setup(&sqlx_pool).await.unwrap();

    let import_store = import::create_storage(&sqlx_pool).await;
    let leaderboard_worker = leaderboard::setup(diesel_pool.clone()).await;
    let import_worker = import::setup(sqlx_pool, diesel_pool.clone()).await;
    let sessions_worker = sessions::setup(diesel_pool).await;

    (
        leaderboard_worker,
        import_worker,
        sessions_worker,
        import_store,
    )
}
