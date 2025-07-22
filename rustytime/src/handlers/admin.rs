use crate::models::heartbeat::{Heartbeat, LanguageCount, ProjectCount};
use crate::models::user::User;
use crate::state::AppState;
use axum::{
    Extension,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use minijinja::context;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct FormattedUser {
    pub id: i32,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: String,
    pub api_key: String,
    pub github_id: i32,
    pub is_admin: bool,
}

#[derive(Serialize, Debug)]
pub struct FormattedDailyActivity {
    pub date: String,
    pub count: i64,
}

#[derive(Serialize, Debug)]
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
}

pub async fn admin_dashboard(
    State(app_state): State<AppState>,
    user: Option<Extension<User>>,
) -> Result<Html<String>, Response> {
    // check if user is an admin
    let current_user = user
        .expect("User should be authenticated since middleware validated authentication")
        .0;

    if !current_user.is_admin() {
        return Err((StatusCode::FORBIDDEN, "Access denied - admin required").into_response());
    }

    // get database connection
    let mut conn = app_state.db_pool.get().map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Database connection error",
        )
            .into_response()
    })?;

    // fetch raw data
    let raw_daily_activity = Heartbeat::get_daily_activity_last_week(&mut conn)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response())?;
    let raw_all_users = User::list_all_users(&mut conn)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response())?;
    let raw_admin_users = User::list_admins(&mut conn)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response())?;

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
        total_users: User::count_total_users(&mut conn)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response())?,
        total_heartbeats: Heartbeat::count_total_heartbeats(&mut conn)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response())?,
        heartbeats_last_hour: Heartbeat::count_heartbeats_last_hour(&mut conn)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response())?,
        heartbeats_last_24h: Heartbeat::count_heartbeats_last_24h(&mut conn)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response())?,
        top_languages: Heartbeat::get_top_languages(&mut conn, 10)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response())?,
        top_projects: Heartbeat::get_top_projects(&mut conn, 10)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response())?,
        daily_activity,
        all_users,
        admin_users,
    };

    let metrics = app_state.metrics.get_metrics();

    let rendered = app_state
        .template_engine
        .render(
            "admin_dashboard.html",
            context! {
                stats => stats,
                current_user => current_user,
                requests_per_second => format!("{:.3}", metrics.requests_per_second),
            },
        )
        .map_err(|err| {
            eprintln!("Template rendering error: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
        })?;

    Ok(Html(rendered))
}
