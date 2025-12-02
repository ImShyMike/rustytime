use crate::db::connection::DbPool;
use crate::models::heartbeat::Heartbeat;
use crate::models::heartbeat::UserDurationRow;
use crate::models::leaderboard::Leaderboard;
use crate::models::leaderboard::NewLeaderboard;
use chrono::{DateTime, Datelike, NaiveDate, Timelike, Utc};
use diesel::prelude::*;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{error, info};

pub struct LeaderboardGenerator {
    db_pool: DbPool,
    next_daily_update: Arc<Mutex<DateTime<Utc>>>,
    next_weekly_update: Arc<Mutex<DateTime<Utc>>>,
    next_all_time_update: Arc<Mutex<DateTime<Utc>>>,
}

impl LeaderboardGenerator {
    pub fn new(db_pool: DbPool) -> Self {
        let now = Utc::now();
        Self {
            db_pool,
            next_daily_update: Arc::new(Mutex::new(next_five_minute_boundary(now))),
            next_weekly_update: Arc::new(Mutex::new(next_top_of_hour(now))),
            next_all_time_update: Arc::new(Mutex::new(next_midnight(now))),
        }
    }

    pub async fn start(self) {
        if let Err(e) = self.generate_full_refresh() {
            error!("Error during initial leaderboard generation: {:?}", e);
        }

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                let tick_started = Instant::now();
                match self.generate_leaderboards_if_needed().await {
                    Ok(_) => tracing::debug!(
                        elapsed_ms = tick_started.elapsed().as_millis() as u64,
                        "Leaderboard refresh tick completed"
                    ),
                    Err(e) => error!("Error generating leaderboards: {:?}", e),
                }
            }
        });
    }

    async fn generate_leaderboards_if_needed(&self) -> Result<(), Box<dyn std::error::Error>> {
        let now = Utc::now();

        let mut conn = self.db_pool.get()?;

        // generate daily leaderboard (every 5 minutes) aligned to midnight
        {
            let next = self.next_daily_update.lock().await;
            if now >= *next {
                let scheduled_time = *next;
                drop(next);
                let today = now.date_naive();
                let start_time = today.and_hms_opt(0, 0, 0).unwrap().and_utc();
                let end_time = (today + chrono::Duration::days(1))
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                self.generate_leaderboard(&mut conn, "daily", today, start_time, end_time)?;
                let mut next = self.next_daily_update.lock().await;
                *next = advance_schedule(scheduled_time, chrono::Duration::minutes(5), now);
            }
        }

        // generate weekly leaderboard (every 1 hour) aligned to hour boundary
        {
            let next = self.next_weekly_update.lock().await;
            if now >= *next {
                let scheduled_time = *next;
                drop(next);
                let week_start = get_week_start(now.date_naive());
                let start_time = week_start.and_hms_opt(0, 0, 0).unwrap().and_utc();
                let end_time = (week_start + chrono::Duration::days(7))
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                self.generate_leaderboard(&mut conn, "weekly", week_start, start_time, end_time)?;
                let mut next = self.next_weekly_update.lock().await;
                *next = advance_schedule(scheduled_time, chrono::Duration::hours(1), now);
            }
        }

        // generate all-time leaderboard (every 24 hours) aligned to midnight
        {
            let next = self.next_all_time_update.lock().await;
            if now >= *next {
                let scheduled_time = *next;
                drop(next);
                let all_time_date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
                let start_time = all_time_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
                let end_time = Utc::now();
                self.generate_leaderboard(
                    &mut conn,
                    "all_time",
                    all_time_date,
                    start_time,
                    end_time,
                )?;
                let mut next = self.next_all_time_update.lock().await;
                *next = advance_schedule(scheduled_time, chrono::Duration::days(1), now);
            }
        }

        // cleanup old leaderboard entries in a single transaction
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            let today = now.date_naive();
            let cutoff_daily = today - chrono::Duration::days(30);
            let cutoff_weekly = today - chrono::Duration::weeks(12);

            Leaderboard::delete_old_daily(conn, cutoff_daily)?;
            Leaderboard::delete_old_weekly(conn, cutoff_weekly)?;

            Ok(())
        })?;
        Ok(())
    }

    fn generate_full_refresh(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut conn = self.db_pool.get()?;
        let now = Utc::now();

        let today = now.date_naive();
        let daily_start = today.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let daily_end = (today + chrono::Duration::days(1))
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        self.generate_leaderboard(&mut conn, "daily", today, daily_start, daily_end)?;

        let week_start = get_week_start(now.date_naive());
        let weekly_start = week_start.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let weekly_end = (week_start + chrono::Duration::days(7))
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();
        self.generate_leaderboard(&mut conn, "weekly", week_start, weekly_start, weekly_end)?;

        let all_time_date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        let all_time_start = all_time_date.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let all_time_end = now;
        self.generate_leaderboard(
            &mut conn,
            "all_time",
            all_time_date,
            all_time_start,
            all_time_end,
        )?;

        Ok(())
    }

    fn generate_leaderboard(
        &self,
        conn: &mut PgConnection,
        period_type: &str,
        period_date: NaiveDate,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        conn.transaction(|conn| {
            let span = tracing::info_span!(
                "generate_leaderboard",
                period = period_type,
                date = %period_date,
                start = %start_time,
                end = %end_time
            );
            let _enter = span.enter();
            let started = Instant::now();

            let results: Vec<UserDurationRow> =
                Heartbeat::get_all_user_durations(conn, start_time, end_time)?;

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

            info!(
                period = period_type,
                rows = results.len(),
                elapsed_ms = started.elapsed().as_millis() as u64,
                "Leaderboard updated"
            );

            Ok(())
        })
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

#[inline(always)]
pub fn get_week_start(date: NaiveDate) -> NaiveDate {
    let weekday = date.weekday().num_days_from_monday();
    date - chrono::Duration::days(weekday as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn week_start_calculates_monday_anchor() {
        let date = NaiveDate::from_ymd_opt(2024, 6, 5).unwrap(); // Wednesday
        let week_start = get_week_start(date);
        assert_eq!(week_start, NaiveDate::from_ymd_opt(2024, 6, 3).unwrap()); // Monday
    }
}
