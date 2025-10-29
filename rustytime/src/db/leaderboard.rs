use crate::db::connection::DbPool;
use crate::models::heartbeat::Heartbeat;
use crate::models::heartbeat::UserDurationRow;
use crate::models::leaderboard::Leaderboard;
use crate::models::leaderboard::NewLeaderboard;
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use diesel::prelude::*;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tracing::error;

pub struct LeaderboardGenerator {
    db_pool: DbPool,
    last_daily_update: Arc<Mutex<DateTime<Utc>>>,
    last_weekly_update: Arc<Mutex<DateTime<Utc>>>,
    last_all_time_update: Arc<Mutex<DateTime<Utc>>>,
}

impl LeaderboardGenerator {
    pub fn new(db_pool: DbPool) -> Self {
        let now = Utc::now();
        Self {
            db_pool,
            last_daily_update: Arc::new(Mutex::new(now - chrono::Duration::minutes(10))),
            last_weekly_update: Arc::new(Mutex::new(now - chrono::Duration::hours(2))),
            last_all_time_update: Arc::new(Mutex::new(now - chrono::Duration::days(2))),
        }
    }

    pub async fn start(self) {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                if let Err(e) = self.generate_leaderboards_if_needed().await {
                    error!("Error generating leaderboards: {:?}", e);
                }
            }
        });
    }

    async fn generate_leaderboards_if_needed(&self) -> Result<(), Box<dyn std::error::Error>> {
        let now = Utc::now();

        let mut conn = self.db_pool.get()?;

        // generate daily leaderboard (every 5 minutes)
        {
            let last = self.last_daily_update.lock().await;
            if now - *last >= chrono::Duration::minutes(5) {
                drop(last);
                let today = now.date_naive();
                let start_time = today.and_hms_opt(0, 0, 0).unwrap().and_utc();
                let end_time = (today + chrono::Duration::days(1))
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                self.generate_leaderboard(&mut conn, "daily", today, start_time, end_time)?;
                *self.last_daily_update.lock().await = now;
            }
        }

        // generate weekly leaderboard (every 1 hour)
        {
            let last = self.last_weekly_update.lock().await;
            if now - *last >= chrono::Duration::hours(1) {
                drop(last);
                let week_start = get_week_start(now.date_naive());
                let start_time = week_start.and_hms_opt(0, 0, 0).unwrap().and_utc();
                let end_time = (week_start + chrono::Duration::days(7))
                    .and_hms_opt(0, 0, 0)
                    .unwrap()
                    .and_utc();
                self.generate_leaderboard(&mut conn, "weekly", week_start, start_time, end_time)?;
                *self.last_weekly_update.lock().await = now;
            }
        }

        // generate all-time leaderboard (every 24 hours)
        {
            let last = self.last_all_time_update.lock().await;
            if now - *last >= chrono::Duration::days(1) {
                drop(last);
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
                *self.last_all_time_update.lock().await = now;
            }
        }

        // cleanup old leaderboard entries
        let today = now.date_naive();
        let cutoff_daily = today - chrono::Duration::days(30);
        let cutoff_weekly = today - chrono::Duration::weeks(12);

        Leaderboard::delete_old_daily(&mut conn, cutoff_daily)?;
        Leaderboard::delete_old_weekly(&mut conn, cutoff_weekly)?;

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

        Ok(())
    }
}

fn get_week_start(date: NaiveDate) -> NaiveDate {
    let weekday = date.weekday().num_days_from_monday();
    date - chrono::Duration::days(weekday as i64)
}
