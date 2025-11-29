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

    let daily = join_with_users(&mut conn, &daily_data);
    let weekly = join_with_users(&mut conn, &weekly_data);
    let all_time = join_with_users(&mut conn, &all_time_data);

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

fn join_with_users(
    conn: &mut PgConnection,
    leaderboard_data: &[Leaderboard],
) -> Vec<LeaderboardEntry> {
    let user_ids: Vec<i32> = leaderboard_data.iter().map(|l| l.user_id).collect();

    let users: Vec<User> = users::table
        .filter(users::id.eq_any(&user_ids))
        .load::<User>(conn)
        .unwrap_or_default();

    let user_map: std::collections::HashMap<i32, &User> = users.iter().map(|u| (u.id, u)).collect();

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
