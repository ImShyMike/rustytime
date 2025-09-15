use crate::models::heartbeat::{Heartbeat, LanguageCount, ProjectCount};
use crate::models::user::User;
use crate::state::AppState;
use crate::{db_query, get_db_conn};
use axum::Json;
use axum::{
    Extension,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Serialize)]
pub struct FormattedUser {
    pub id: i32,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: String,
    pub api_key: String,
    pub github_id: i32,
    pub is_admin: bool,
}

#[derive(Serialize)]
pub struct FormattedDailyActivity {
    pub date: String,
    pub count: i64,
}

#[derive(Serialize)]
pub struct AdminStats {
    pub total_users: i64,
    pub total_heartbeats: i64,
    pub heartbeats_last_hour: i64,
    pub heartbeats_last_24h: i64,
    pub top_languages: Vec<LanguageCount>,
    pub top_projects: Vec<ProjectCount>,
    pub daily_activity: Vec<FormattedDailyActivity>,
    pub all_users: Vec<FormattedUser>,
    pub admin_users: Vec<FormattedUser>,
    pub requests_per_second: String,
}

#[derive(Serialize)]
pub struct AdminDashboardResponse {
    pub stats: AdminStats,
    pub current_user: User,
}

pub async fn admin_dashboard(
    State(app_state): State<AppState>,
    user: Option<Extension<User>>,
) -> Result<Json<AdminDashboardResponse>, Response> {
    // check if user is an admin
    let current_user = user
        .expect("User should be authenticated since middleware validated authentication")
        .0;

    if !current_user.is_admin() {
        return Err((StatusCode::FORBIDDEN, "No permission").into_response());
    }

    // get database connection
    let mut conn = get_db_conn!(app_state);

    // fetch raw data
    let raw_daily_activity = db_query!(
        Heartbeat::get_daily_activity_last_week(&mut conn),
        "Failed to fetch daily activity"
    );
    let raw_all_users = db_query!(User::list_all_users(&mut conn), "Failed to fetch users");
    let raw_admin_users = db_query!(User::list_admins(&mut conn), "Failed to fetch admin users");

    // convert to formatted versions
    let daily_activity: Vec<FormattedDailyActivity> = raw_daily_activity
        .into_iter()
        .map(|activity| FormattedDailyActivity {
            date: activity.date.format("%m-%d").to_string(),
            count: activity.count,
        })
        .collect();

    let all_users: Vec<FormattedUser> = raw_all_users
        .into_iter()
        .map(|user| FormattedUser {
            id: user.id,
            name: user.name,
            avatar_url: user.avatar_url,
            created_at: user.created_at.format("%Y-%m-%d").to_string(),
            api_key: user.api_key.to_string(),
            github_id: user.github_id,
            is_admin: user.is_admin,
        })
        .collect();

    let admin_users: Vec<FormattedUser> = raw_admin_users
        .into_iter()
        .map(|user| FormattedUser {
            id: user.id,
            name: user.name,
            avatar_url: user.avatar_url,
            created_at: user.created_at.format("%Y-%m-%d").to_string(),
            api_key: user.api_key.to_string(),
            github_id: user.github_id,
            is_admin: user.is_admin,
        })
        .collect();

    // fetch all statistics
    let stats = AdminStats {
        total_users: db_query!(User::count_total_users(&mut conn)),
        total_heartbeats: db_query!(Heartbeat::count_total_heartbeats(&mut conn)),
        heartbeats_last_hour: db_query!(Heartbeat::count_heartbeats_last_hour(&mut conn)),
        heartbeats_last_24h: db_query!(Heartbeat::count_heartbeats_last_24h(&mut conn)),
        top_languages: db_query!(Heartbeat::get_top_languages(&mut conn, 10)),
        top_projects: db_query!(Heartbeat::get_top_projects(&mut conn, 10)),
        daily_activity,
        all_users,
        admin_users,
        requests_per_second: format!("{:.3}", app_state.metrics.get_metrics().requests_per_second),
    };

    Ok(Json(AdminDashboardResponse {
        stats,
        current_user,
    }))
}
