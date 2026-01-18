use crate::db::leaderboard::get_week_start;
use crate::models::leaderboard::{Leaderboard, LeaderboardEntry};
use crate::models::user::User;
use crate::schema::users;
use crate::state::AppState;
use crate::{db_query, get_db_conn};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use chrono::{DateTime, NaiveDate, Utc};
use diesel::prelude::*;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct LeaderboardResponse {
    daily: LeaderboardData,
    weekly: LeaderboardData,
    all_time: LeaderboardData,
}

#[derive(Serialize, JsonSchema)]
struct LeaderboardData {
    generated_at: DateTime<Utc>,
    entries: Vec<LeaderboardEntry>,
}

pub async fn leaderboard_page(
    State(app_state): State<AppState>,
) -> Result<Json<LeaderboardResponse>, Response> {
    let mut conn = get_db_conn!(app_state);

    let today = Utc::now().date_naive();
    let week_start = get_week_start(today);
    let all_time_date = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();

    let daily_data = db_query!(
        Leaderboard::get_by_period(&mut conn, "daily", today),
        "Database error getting daily leaderboard"
    );

    let weekly_data = db_query!(
        Leaderboard::get_by_period(&mut conn, "weekly", week_start),
        "Database error getting weekly leaderboard"
    );

    let all_time_data = db_query!(
        Leaderboard::get_by_period(&mut conn, "all_time", all_time_date),
        "Database error getting all-time leaderboard"
    );

    let mut all_user_ids: Vec<i32> =
        Vec::with_capacity(daily_data.len() + weekly_data.len() + all_time_data.len());
    all_user_ids.extend(daily_data.iter().map(|l| l.user_id));
    all_user_ids.extend(weekly_data.iter().map(|l| l.user_id));
    all_user_ids.extend(all_time_data.iter().map(|l| l.user_id));
    all_user_ids.sort_unstable();
    all_user_ids.dedup();

    let all_users: Vec<User> = users::table
        .filter(users::id.eq_any(&all_user_ids))
        .load::<User>(&mut conn)
        .unwrap_or_default();

    let user_map: std::collections::HashMap<i32, &User> =
        all_users.iter().map(|u| (u.id, u)).collect();

    let daily = map_leaderboard_entries(&daily_data, &user_map);
    let weekly = map_leaderboard_entries(&weekly_data, &user_map);
    let all_time = map_leaderboard_entries(&all_time_data, &user_map);

    Ok(Json(LeaderboardResponse {
        daily: LeaderboardData {
            generated_at: daily_data
                .first()
                .map(|l| l.updated_at)
                .unwrap_or_else(Utc::now),
            entries: daily,
        },
        weekly: LeaderboardData {
            generated_at: weekly_data
                .first()
                .map(|l| l.updated_at)
                .unwrap_or_else(Utc::now),
            entries: weekly,
        },
        all_time: LeaderboardData {
            generated_at: all_time_data
                .first()
                .map(|l| l.updated_at)
                .unwrap_or_else(Utc::now),
            entries: all_time,
        },
    }))
}

fn map_leaderboard_entries(
    leaderboard_data: &[Leaderboard],
    user_map: &std::collections::HashMap<i32, &User>,
) -> Vec<LeaderboardEntry> {
    leaderboard_data
        .iter()
        .filter_map(|l| {
            user_map.get(&l.user_id).map(|user| LeaderboardEntry {
                user_id: l.user_id,
                user_name: user.name.clone(),
                avatar_url: user.avatar_url.clone(),
                total_seconds: l.total_seconds,
                rank: l.rank,
            })
        })
        .collect()
}
