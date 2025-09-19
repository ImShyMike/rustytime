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
    pub requests_per_second: f64,
    pub top_languages: Vec<LanguageCount>,
    pub top_projects: Vec<ProjectCount>,
    pub daily_activity: Vec<FormattedDailyActivity>,
    pub all_users: Vec<User>,
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
    let all_users = db_query!(User::list_all_users(&mut conn), "Failed to fetch users");

    // convert to formatted versions
    let daily_activity: Vec<FormattedDailyActivity> = raw_daily_activity
        .into_iter()
        .map(|activity| FormattedDailyActivity {
            date: activity.date.format("%m-%d").to_string(),
            count: activity.count,
        })
        .collect();

    // fetch all statistics
    let stats = AdminStats {
        total_users: db_query!(User::count_total_users(&mut conn, false)),
        total_heartbeats: db_query!(Heartbeat::count_total_heartbeats(&mut conn)),
        heartbeats_last_hour: db_query!(Heartbeat::count_heartbeats_last_hour(&mut conn)),
        heartbeats_last_24h: db_query!(Heartbeat::count_heartbeats_last_24h(&mut conn)),
        requests_per_second: (app_state.metrics.get_metrics().requests_per_second * 1000.0).round()
            / 1000.0,
        top_languages: db_query!(Heartbeat::get_top_languages(&mut conn, 10)),
        top_projects: db_query!(Heartbeat::get_top_projects(&mut conn, 10)),
        daily_activity,
        all_users,
    };

    Ok(Json(AdminDashboardResponse {
        stats,
        current_user,
    }))
}
