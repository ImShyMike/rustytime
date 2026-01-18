use std::time::Duration;

use apalis::{
    layers::WorkerBuilderExt,
    prelude::{
        BackoffConfig, BoxDynError, Codec, Data, IntervalStrategy, StrategyBuilder, WorkerBuilder,
    },
};
use apalis_postgres::PostgresStorage;
use chrono::{DateTime, Datelike, NaiveDate, Timelike, Utc};
use futures::{FutureExt, TryFutureExt};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sqlx::PgPool;
use tokio::signal::ctrl_c;

use axum_prometheus::metrics;
use diesel::Connection;

use crate::db::connection::DbPool;
use crate::models::heartbeat::Heartbeat;
use crate::models::leaderboard::{Leaderboard, NewLeaderboard};

#[derive(Clone)]
struct JsonCodec;

impl<T: Serialize + DeserializeOwned> Codec<T> for JsonCodec {
    type Compact = Vec<u8>;
    type Error = serde_json::Error;

    fn encode(input: &T) -> Result<Vec<u8>, Self::Error> {
        serde_json::to_vec(input)
    }

    fn decode(compact: &Vec<u8>) -> Result<T, Self::Error> {
        serde_json::from_slice(compact)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LeaderboardJob {
    pub period_type: String,
    pub period_date: NaiveDate,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
}

impl LeaderboardJob {
    pub fn daily(date: NaiveDate) -> Self {
        let start_time = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end_time = (date + chrono::Duration::days(1))
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        Self {
            period_type: "daily".to_string(),
            period_date: date,
            start_time,
            end_time,
        }
    }

    pub fn weekly(date: NaiveDate) -> Self {
        let week_start = get_week_start(date);
        let start_time = week_start.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end_time = (week_start + chrono::Duration::days(7))
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        Self {
            period_type: "weekly".to_string(),
            period_date: week_start,
            start_time,
            end_time,
        }
    }

    pub fn all_time() -> Self {
        let all_time_date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        let start_time = all_time_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let end_time = Utc::now();
        Self {
            period_type: "all_time".to_string(),
            period_date: all_time_date,
            start_time,
            end_time,
        }
    }
}

async fn regenerate_leaderboard(
    job: LeaderboardJob,
    pool: Data<DbPool>,
) -> Result<String, BoxDynError> {
    let started = std::time::Instant::now();
    let period_type = job.period_type.clone();

    tracing::info!(
        period = %period_type,
        date = %job.period_date,
        "Starting leaderboard regeneration"
    );

    let result = (|| {
        let mut conn = pool.get()?;

        conn.build_transaction().run(|conn| {
            let results = Heartbeat::get_all_user_durations(conn, job.start_time, job.end_time)?;

            let leaderboard_entries: Vec<NewLeaderboard> = results
                .iter()
                .enumerate()
                .map(|(idx, row)| NewLeaderboard {
                    user_id: row.user_id,
                    period_type: job.period_type.clone(),
                    period_date: job.period_date,
                    total_seconds: row.total_seconds,
                    rank: (idx + 1) as i32,
                })
                .collect();

            Leaderboard::upsert_batch(conn, leaderboard_entries)?;

            Ok::<_, diesel::result::Error>(results.len())
        })?;

        Ok::<_, BoxDynError>(())
    })();

    let elapsed = started.elapsed();
    let status = if result.is_ok() { "ok" } else { "error" };

    metrics::counter!("leaderboard_jobs_total", "period" => period_type.clone(), "status" => status).increment(1);
    metrics::histogram!("leaderboard_job_duration_seconds", "period" => period_type.clone())
        .record(elapsed.as_secs_f64());

    tracing::info!(
        period = %period_type,
        elapsed_ms = elapsed.as_millis() as u64,
        status = status,
        "Leaderboard regeneration completed"
    );

    result?;

    Ok(format!(
        "Leaderboard {} regenerated in {:?}",
        period_type, elapsed
    ))
}

async fn schedule_leaderboard_jobs(
    mut storage: PostgresStorage<LeaderboardJob, Vec<u8>, JsonCodec, apalis_postgres::PgNotify>,
    pool: DbPool,
) {
    use apalis::prelude::TaskSink;

    let today = Utc::now().date_naive();

    let initial_jobs = vec![
        LeaderboardJob::daily(today),
        LeaderboardJob::weekly(today),
        LeaderboardJob::all_time(),
    ];

    for job in initial_jobs {
        match storage.push(job.clone()).await {
            Ok(_) => tracing::info!(period = %job.period_type, "Queued initial leaderboard job"),
            Err(e) => {
                tracing::error!(period = %job.period_type, error = ?e, "Failed to queue initial job")
            }
        }
    }

    let now = Utc::now();
    let mut next_daily = next_five_minute_boundary(now);
    let mut next_weekly = next_top_of_hour(now);
    let mut next_all_time = next_midnight(now);
    let mut next_cleanup = next_midnight(now);

    loop {
        let now = Utc::now();

        let sleep_until = [next_daily, next_weekly, next_all_time, next_cleanup]
            .into_iter()
            .min()
            .unwrap();

        if sleep_until > now {
            let sleep_duration = (sleep_until - now)
                .to_std()
                .unwrap_or(Duration::from_secs(1));
            tokio::time::sleep(sleep_duration).await;
        }

        let now = Utc::now();

        if now >= next_daily {
            let today = now.date_naive();
            let job = LeaderboardJob::daily(today);
            if let Err(e) = storage.push(job).await {
                tracing::error!(error = ?e, "Failed to queue daily leaderboard job");
            }
            next_daily = advance_schedule(next_daily, chrono::Duration::minutes(5), now);
        }

        if now >= next_weekly {
            let today = now.date_naive();
            let job = LeaderboardJob::weekly(today);
            if let Err(e) = storage.push(job).await {
                tracing::error!(error = ?e, "Failed to queue weekly leaderboard job");
            }
            next_weekly = advance_schedule(next_weekly, chrono::Duration::hours(1), now);
        }

        if now >= next_all_time {
            let job = LeaderboardJob::all_time();
            if let Err(e) = storage.push(job).await {
                tracing::error!(error = ?e, "Failed to queue all-time leaderboard job");
            }
            next_all_time = advance_schedule(next_all_time, chrono::Duration::days(1), now);
        }

        if now >= next_cleanup {
            if let Err(e) = cleanup_old_entries(&pool) {
                tracing::error!(error = ?e, "Failed to cleanup old leaderboard entries");
            }
            next_cleanup = advance_schedule(next_cleanup, chrono::Duration::days(1), now);
        }
    }
}

fn next_five_minute_boundary(now: DateTime<Utc>) -> DateTime<Utc> {
    const STEP_SECS: i64 = 300;
    const DAY_SECS: i64 = 86_400;

    let secs = now.time().num_seconds_from_midnight() as i64;
    let next_secs = ((secs / STEP_SECS) + 1) * STEP_SECS;
    let (target_day, secs_in_day) = if next_secs >= DAY_SECS {
        (
            now.date_naive() + chrono::Duration::days(1),
            next_secs - DAY_SECS,
        )
    } else {
        (now.date_naive(), next_secs)
    };

    let midnight = target_day
        .and_hms_opt(0, 0, 0)
        .expect("valid midnight")
        .and_utc();
    midnight + chrono::Duration::seconds(secs_in_day)
}

fn next_top_of_hour(now: DateTime<Utc>) -> DateTime<Utc> {
    (now + chrono::Duration::hours(1))
        .with_minute(0)
        .and_then(|dt| dt.with_second(0))
        .and_then(|dt| dt.with_nanosecond(0))
        .expect("valid next hour")
}

fn next_midnight(now: DateTime<Utc>) -> DateTime<Utc> {
    (now.date_naive() + chrono::Duration::days(1))
        .and_hms_opt(0, 0, 0)
        .expect("valid midnight")
        .and_utc()
}

fn advance_schedule(
    scheduled_time: DateTime<Utc>,
    step: chrono::Duration,
    reference: DateTime<Utc>,
) -> DateTime<Utc> {
    let mut next = scheduled_time + step;
    while next <= reference {
        next += step;
    }
    next
}

fn cleanup_old_entries(pool: &DbPool) -> Result<(), diesel::result::Error> {
    let mut conn = pool.get().map_err(|e| {
        tracing::error!(error = ?e, "Failed to get connection for cleanup");
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(e.to_string()),
        )
    })?;

    let today = Utc::now().date_naive();
    let cutoff_daily = today - chrono::Duration::days(30);
    let cutoff_weekly = today - chrono::Duration::weeks(12);

    conn.transaction(|conn| {
        let daily_deleted = Leaderboard::delete_old_daily(conn, cutoff_daily)?;
        let weekly_deleted = Leaderboard::delete_old_weekly(conn, cutoff_weekly)?;

        tracing::info!(
            daily_deleted,
            weekly_deleted,
            "Cleaned up old leaderboard entries"
        );

        Ok(())
    })
}

type LeaderboardStore =
    PostgresStorage<LeaderboardJob, Vec<u8>, JsonCodec, apalis_postgres::PgNotify>;

pub async fn setup(
    sqlx_pool: PgPool,
    diesel_pool: DbPool,
) -> impl std::future::Future<Output = ()> {
    let storage_config = apalis_postgres::Config::new("leaderboard_jobs").with_poll_interval(
        StrategyBuilder::new()
            .apply(
                IntervalStrategy::new(Duration::from_secs(5))
                    .with_backoff(BackoffConfig::default()),
            )
            .build(),
    );

    let leaderboard_store: LeaderboardStore =
        PostgresStorage::new_with_notify(&sqlx_pool, &storage_config).with_codec::<JsonCodec>();

    let scheduler_storage = leaderboard_store.clone();
    let scheduler_pool = diesel_pool.clone();
    tokio::spawn(schedule_leaderboard_jobs(scheduler_storage, scheduler_pool));

    WorkerBuilder::new("leaderboard-worker")
        .backend(leaderboard_store)
        .enable_tracing()
        .catch_panic()
        .concurrency(2)
        .data(diesel_pool)
        .build(regenerate_leaderboard)
        .run_until(ctrl_c())
        .map_err(|e| tracing::error!("Worker error: {}", e))
        .map(|_| ())
}

#[inline(always)]
pub fn get_week_start(date: NaiveDate) -> NaiveDate {
    let weekday = date.weekday().num_days_from_monday();
    date - chrono::Duration::days(weekday as i64)
}
