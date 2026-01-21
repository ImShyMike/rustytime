use crate::models::heartbeat::{TimeRange, UsageStat};
use crate::models::user::User;
use crate::state::AppState;
use crate::utils::cache::{CachedDashboardStats, DashboardCacheKey};
use crate::utils::session::SessionManager;
use crate::utils::time::{TimeFormat, human_readable_duration};
use crate::{db_query, get_db_conn, models::heartbeat::Heartbeat};
use aide::NoApi;
use axum::{
    Extension,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;

#[derive(Deserialize, JsonSchema)]
pub struct DashboardQuery {
    #[serde(default)]
    pub range: TimeRange,
}

#[derive(Serialize, JsonSchema)]
pub struct DashboardResponse {
    avatar_url: String,
    username: String,
    user_id: i32,
    github_id: i64,
    created_at: String,
    expires_at: String,
    total_heartbeats: i64,
    human_readable_total: String,
    admin_level: i16,
    dev_mode: bool,
    range: String,
    projects: Vec<UsageStat>,
    editors: Vec<UsageStat>,
    operating_systems: Vec<UsageStat>,
    languages: Vec<UsageStat>,
}

/// Handler for the dashboard page
pub async fn dashboard(
    State(app_state): State<AppState>,
    cookies: NoApi<Cookies>,
    user: Option<Extension<User>>,
    Query(query): Query<DashboardQuery>,
) -> Result<Json<DashboardResponse>, Response> {
    let cookies = cookies.0;
    // check if user is authenticated
    if user.is_none() {
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "User should be authenticated since middleware validated authentication",
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
            "User should be authenticated since middleware validated authentication",
        )
            .into_response());
    };

    let user_timezone = user.timezone.clone();
    let cache_key = DashboardCacheKey {
        user_id: session_data.user_id,
        range: query.range,
        timezone: user_timezone.clone(),
    };

    let cached = app_state.cache.dashboard.get(&cache_key);
    let (total_heartbeats, dashboard_stats) = if let Some(cached) = cached {
        (cached.heartbeat_count, cached.stats)
    } else {
        let mut conn = get_db_conn!(app_state);

        let total_heartbeats = db_query!(
            Heartbeat::get_user_heartbeat_count_by_range(
                &mut conn,
                session_data.user_id,
                query.range,
                &user_timezone
            ),
            "Database error getting heartbeat count"
        );

        let dashboard_stats = db_query!(
            Heartbeat::get_dashboard_stats_by_range(
                &mut conn,
                session_data.user_id,
                query.range,
                &user_timezone
            ),
            "Database error getting dashboard stats"
        );

        app_state.cache.dashboard.insert(
            cache_key,
            CachedDashboardStats {
                stats: dashboard_stats.clone(),
                heartbeat_count: total_heartbeats,
            },
        );

        (total_heartbeats, dashboard_stats)
    };

    Ok(Json(DashboardResponse {
        avatar_url: user.avatar_url,
        username: user.name,
        user_id: user.id,
        github_id: session_data.github_user_id,
        created_at: user.created_at.to_rfc3339(),
        expires_at: session_data.expires_at.to_rfc3339(),
        total_heartbeats,
        human_readable_total: human_readable_duration(
            dashboard_stats.total_time,
            TimeFormat::NoDays,
        )
        .human_readable,
        admin_level: user.admin_level,
        dev_mode: cfg!(debug_assertions),
        range: query.range.as_str().to_string(),
        projects: dashboard_stats.top_projects,
        editors: dashboard_stats.top_editors,
        operating_systems: dashboard_stats.top_oses,
        languages: dashboard_stats.top_languages,
    }))
}
