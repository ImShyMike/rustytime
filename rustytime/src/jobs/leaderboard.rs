use std::str::FromStr;

use apalis::{
    layers::WorkerBuilderExt,
    prelude::{Data, WorkerBuilder},
};
use apalis_cron::{CronStream, Tick};
use chrono::{NaiveDate, Utc};
use cron::Schedule;
use tokio::signal::ctrl_c;

use axum_prometheus::metrics;
use diesel::Connection;

use crate::db::connection::DbPool;
use crate::models::heartbeat::Heartbeat;
use crate::models::leaderboard::{Leaderboard, NewLeaderboard};
use crate::utils::time::get_week_start;

const DAILY_RETENTION_DAYS: i64 = 30;
const WEEKLY_RETENTION_WEEKS: i64 = 12;

async fn regenerate_daily_leaderboard(_tick: Tick, pool: Data<DbPool>) {
    let started = std::time::Instant::now();
    let today = Utc::now().date_naive();

    tracing::debug!(period = "daily", date = %today, "Starting leaderboard regeneration");

    let result = regenerate_leaderboard_period(&pool, "daily", today);

    let elapsed = started.elapsed();
    let status = if result.is_ok() { "ok" } else { "error" };

    metrics::counter!("leaderboard_jobs_total", "period" => "daily", "status" => status)
        .increment(1);
    metrics::histogram!("leaderboard_job_duration_seconds", "period" => "daily")
        .record(elapsed.as_secs_f64());

    tracing::debug!(
        period = "daily",
        elapsed_ms = elapsed.as_millis() as u64,
        status = status,
        "Leaderboard regeneration completed"
    );
}

async fn regenerate_weekly_leaderboard(_tick: Tick, pool: Data<DbPool>) {
    let started = std::time::Instant::now();
    let today = Utc::now().date_naive();
    let week_start = get_week_start(today);

    tracing::debug!(period = "weekly", date = %week_start, "Starting leaderboard regeneration");

    let result = regenerate_leaderboard_period(&pool, "weekly", week_start);

    let elapsed = started.elapsed();
    let status = if result.is_ok() { "ok" } else { "error" };

    metrics::counter!("leaderboard_jobs_total", "period" => "weekly", "status" => status)
        .increment(1);
    metrics::histogram!("leaderboard_job_duration_seconds", "period" => "weekly")
        .record(elapsed.as_secs_f64());

    tracing::debug!(
        period = "weekly",
        elapsed_ms = elapsed.as_millis() as u64,
        status = status,
        "Leaderboard regeneration completed"
    );
}

async fn regenerate_all_time_leaderboard(_tick: Tick, pool: Data<DbPool>) {
    let started = std::time::Instant::now();
    let all_time_date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();

    tracing::debug!(period = "all_time", "Starting leaderboard regeneration");

    let result = regenerate_leaderboard_period(&pool, "all_time", all_time_date);

    let elapsed = started.elapsed();
    let status = if result.is_ok() { "ok" } else { "error" };

    metrics::counter!("leaderboard_jobs_total", "period" => "all_time", "status" => status)
        .increment(1);
    metrics::histogram!("leaderboard_job_duration_seconds", "period" => "all_time")
        .record(elapsed.as_secs_f64());

    tracing::debug!(
        period = "all_time",
        elapsed_ms = elapsed.as_millis() as u64,
        status = status,
        "Leaderboard regeneration completed"
    );
}

fn regenerate_leaderboard_period(
    pool: &DbPool,
    period_type: &str,
    period_date: NaiveDate,
) -> Result<(), diesel::result::Error> {
    let (start_time, end_time) = match period_type {
        "daily" => {
            let start = period_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
            let end = (period_date + chrono::Duration::days(1))
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
            (start, end)
        }
        "weekly" => {
            let week_start = get_week_start(period_date);
            let start = week_start.and_hms_opt(0, 0, 0).unwrap().and_utc();
            let end = (week_start + chrono::Duration::days(7))
                .and_hms_opt(0, 0, 0)
                .unwrap()
                .and_utc();
            (start, end)
        }
        "all_time" => {
            let start = period_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
            let end = Utc::now();
            (start, end)
        }
        _ => return Ok(()),
    };

    let mut conn = pool.get().map_err(|e| {
        tracing::error!(error = ?e, "Failed to get connection");
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(e.to_string()),
        )
    })?;

    conn.build_transaction().run(|conn| {
        let results = Heartbeat::get_all_user_durations(conn, start_time, end_time)?;

        let leaderboard_entries: Vec<NewLeaderboard> = results
            .iter()
            .enumerate()
            .map(|(idx, row)| NewLeaderboard {
                user_id: row.user_id,
                period_type: period_type.to_string(),
                period_date,
                total_seconds: row.total_seconds,
                rank: (idx + 1) as i32,
            })
            .collect();

        Leaderboard::upsert_batch(conn, leaderboard_entries)?;

        Ok(())
    })
}

fn cleanup_old_entries(pool: &DbPool) -> Result<(), diesel::result::Error> {
    let mut conn = pool.get().map_err(|e| {
        tracing::error!(error = ?e, "Failed to get connection for leaderboard cleanup");
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(e.to_string()),
        )
    })?;

    let today = Utc::now().date_naive();
    let cutoff_daily = today - chrono::Duration::days(DAILY_RETENTION_DAYS);
    let cutoff_weekly = today - chrono::Duration::weeks(WEEKLY_RETENTION_WEEKS);

    conn.transaction(|conn| {
        let daily_deleted = Leaderboard::delete_old_daily(conn, cutoff_daily)?;
        let weekly_deleted = Leaderboard::delete_old_weekly(conn, cutoff_weekly)?;

        tracing::debug!(
            daily_deleted,
            weekly_deleted,
            "Cleaned up old leaderboard entries"
        );

        Ok(())
    })
}

async fn run_cleanup(_tick: Tick, pool: Data<DbPool>) {
    if let Err(e) = cleanup_old_entries(&pool) {
        tracing::error!(error = ?e, "Failed to cleanup old leaderboard entries");
    }
}

pub async fn setup(diesel_pool: DbPool) -> impl std::future::Future<Output = ()> {
    let daily_schedule = Schedule::from_str("0 */5 * * * *").expect("valid cron: every 5 minutes");
    let weekly_schedule = Schedule::from_str("0 0 * * * *").expect("valid cron: every hour");
    let all_time_schedule =
        Schedule::from_str("0 0 0 * * *").expect("valid cron: daily at midnight");
    let cleanup_schedule =
        Schedule::from_str("0 10 0 * * *").expect("valid cron: daily at 00:10 AM");

    let daily_pool = diesel_pool.clone();
    let weekly_pool = diesel_pool.clone();
    let all_time_pool = diesel_pool.clone();
    let cleanup_pool = diesel_pool;

    let daily_worker = WorkerBuilder::new("leaderboard-daily")
        .backend(CronStream::new(daily_schedule))
        .enable_tracing()
        .catch_panic()
        .data(daily_pool)
        .build(regenerate_daily_leaderboard);

    let weekly_worker = WorkerBuilder::new("leaderboard-weekly")
        .backend(CronStream::new(weekly_schedule))
        .enable_tracing()
        .catch_panic()
        .data(weekly_pool)
        .build(regenerate_weekly_leaderboard);

    let all_time_worker = WorkerBuilder::new("leaderboard-all-time")
        .backend(CronStream::new(all_time_schedule))
        .enable_tracing()
        .catch_panic()
        .data(all_time_pool)
        .build(regenerate_all_time_leaderboard);

    let cleanup_worker = WorkerBuilder::new("leaderboard-cleanup")
        .backend(CronStream::new(cleanup_schedule))
        .enable_tracing()
        .catch_panic()
        .data(cleanup_pool)
        .build(run_cleanup);

    async move {
        tokio::select! {
            _ = daily_worker.run() => {}
            _ = weekly_worker.run() => {}
            _ = all_time_worker.run() => {}
            _ = cleanup_worker.run() => {}
            _ = ctrl_c() => {
                tracing::info!("Shutting down leaderboard workers");
            }
        }
    }
}
