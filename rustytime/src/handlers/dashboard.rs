use crate::models::heartbeat::UsageStat;
use crate::models::user::User;
use crate::state::AppState;
use crate::utils::session::SessionManager;
use crate::utils::time::human_readable_duration;
use crate::{db_query, get_db_conn, models::heartbeat::Heartbeat};
use axum::{
    Extension,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Serialize;
use tower_cookies::Cookies;

#[derive(Serialize)]
pub struct DashboardResponse {
    avatar_url: String,
    user_name: String,
    github_id: i64,
    created_at: String,
    expires_at: String,
    api_key: String,
    total_heartbeats: i64,
    formatted_time: String,
    top_projects: Vec<UsageStat>,
    top_editors: Vec<UsageStat>,
    top_os: Vec<UsageStat>,
    top_languages: Vec<UsageStat>,
    is_admin: bool,
    dev_mode: bool,
}

/// Handler for the dashboard page (will likely be done using svelteKit later)
pub async fn dashboard(
    State(app_state): State<AppState>,
    cookies: Cookies,
    user: Option<Extension<User>>,
) -> Result<Json<DashboardResponse>, Response> {
    // check if user is authenticated
    if user.is_none() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "This should not happen D:",
        )
            .into_response());
    }
    let user = user.unwrap().0;

    // get user's session info
    let session_id = SessionManager::get_session_from_cookies(&cookies)
        .expect("Session should exist since middleware validated authentication");

    let Some(session_data) = db_query!(
        SessionManager::validate_session(&app_state.db_pool, session_id).await,
        "Session validation error"
    ) else {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "This should not happen D:",
        )
            .into_response());
    };

    let mut conn = get_db_conn!(app_state);

    // get heartbeat count
    let total_heartbeats = db_query!(
        Heartbeat::get_user_heartbeat_count(&mut conn, session_data.user_id),
        "Database error getting heartbeat count"
    );

    // get dashboard stats
    let dashboard_stats = db_query!(
        Heartbeat::get_dashboard_stats(&mut conn, session_data.user_id),
        "Database error getting dashboard stats"
    );

    Ok(Json(DashboardResponse {
        avatar_url: user.avatar_url.as_deref().unwrap_or("").to_string(),
        user_name: user.name.as_deref().unwrap_or("Unknown").to_string(),
        github_id: session_data.github_user_id,
        created_at: user.created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        expires_at: session_data
            .expires_at
            .format("%Y-%m-%d %H:%M:%S UTC")
            .to_string(),
        api_key: user.api_key.to_string(),
        total_heartbeats: total_heartbeats,
        formatted_time: human_readable_duration(dashboard_stats.total_time, true).human_readable,
        top_projects: dashboard_stats.top_projects,
        top_editors: dashboard_stats.top_editors,
        top_os: dashboard_stats.top_oses,
        top_languages: dashboard_stats.top_languages,
        is_admin: user.is_admin,
        dev_mode: cfg!(debug_assertions),
    }))
}
