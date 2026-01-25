use chrono::NaiveDate;
use moka::sync::Cache;
use std::sync::Arc;
use std::time::Duration;

use crate::handlers::page::projects::Project;
use crate::models::heartbeat::{DailyActivity, DashboardStats, TimeRange};
use crate::models::leaderboard::Leaderboard;
use crate::models::user::User;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DashboardCacheKey {
    pub user_id: i32,
    pub range: TimeRange,
    pub timezone: String,
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
    pub daily_activity: Vec<DailyActivity>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct HeartbeatProjectCacheKey {
    pub user_id: i32,
    pub project_name: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ProjectsCacheKey {
    pub user_id: i32,
}

#[derive(Clone)]
pub struct AppCache {
    pub dashboard: Arc<Cache<DashboardCacheKey, CachedDashboardStats>>,
    pub projects: Arc<Cache<ProjectsCacheKey, Vec<Project>>>,
    pub leaderboard: Arc<Cache<LeaderboardCacheKey, CachedLeaderboard>>,
    pub admin: Arc<Cache<(), CachedAdminStats>>,
}

impl AppCache {
    pub fn new() -> Self {
        Self {
            dashboard: Arc::new(
                Cache::builder()
                    .max_capacity(1_000)
                    .time_to_live(Duration::from_secs(300)) // 5 minute TTL
                    .support_invalidation_closures()
                    .build(),
            ),
            projects: Arc::new(
                Cache::builder()
                    .max_capacity(1_000)
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
                    .time_to_live(Duration::from_secs(300)) // 5 minute TTL
                    .build(),
            ),
        }
    }

    pub fn invalidate_user_dashboard(&self, user_id: i32) {
        let _ = self
            .dashboard
            .invalidate_entries_if(move |key, _| key.user_id == user_id);
    }

    pub fn invalidate_user_projects(&self, user_id: i32) {
        self.projects.invalidate(&ProjectsCacheKey { user_id });
    }
}

impl Default for AppCache {
    fn default() -> Self {
        Self::new()
    }
}
