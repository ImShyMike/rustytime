use chrono::NaiveDate;
use moka::sync::Cache;
use std::sync::Arc;
use std::time::Duration;

use crate::models::heartbeat::{DailyActivity, DashboardStats, TimeRange};
use crate::models::leaderboard::Leaderboard;
use crate::models::user::User;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DashboardCacheKey {
    pub user_id: i32,
    pub range: TimeRange,
}

#[derive(Clone)]
pub struct CachedDashboardStats {
    pub stats: DashboardStats,
    pub heartbeat_count: i64,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LeaderboardCacheKey {
    pub period_type: String,
    pub period_date: NaiveDate,
}

#[derive(Clone)]
pub struct CachedLeaderboard {
    pub entries: Vec<Leaderboard>,
    pub users: Vec<User>,
}

#[derive(Clone)]
pub struct CachedAdminStats {
    pub total_users: i64,
    pub total_heartbeats: i64,
    pub heartbeats_last_hour: i64,
    pub heartbeats_last_24h: i64,
    pub daily_activity: Vec<DailyActivity>,
}

#[derive(Clone)]
pub struct AppCache {
    pub dashboard: Arc<Cache<DashboardCacheKey, CachedDashboardStats>>,
    pub leaderboard: Arc<Cache<LeaderboardCacheKey, CachedLeaderboard>>,
    pub admin: Arc<Cache<(), CachedAdminStats>>,
}

impl AppCache {
    pub fn new() -> Self {
        Self {
            dashboard: Arc::new(
                Cache::builder()
                    .max_capacity(10_000)
                    .time_to_live(Duration::from_secs(300)) // 5 minute TTL
                    .build(),
            ),
            leaderboard: Arc::new(
                Cache::builder()
                    .max_capacity(100)
                    .time_to_live(Duration::from_secs(60)) // 1 minute TTL
                    .build(),
            ),
            admin: Arc::new(
                Cache::builder()
                    .max_capacity(1)
                    .time_to_live(Duration::from_secs(30)) // 30 second TTL
                    .build(),
            ),
        }
    }

    pub fn invalidate_user_dashboard(&self, user_id: i32) {
        for range in [
            TimeRange::Day,
            TimeRange::Week,
            TimeRange::Month,
            TimeRange::All,
        ] {
            self.dashboard
                .invalidate(&DashboardCacheKey { user_id, range });
        }
    }
}

impl Default for AppCache {
    fn default() -> Self {
        Self::new()
    }
}
