use crate::db_query;
use crate::models::heartbeat::Heartbeat;
use crate::models::heartbeat::{TimeRange, UsageStat};
use crate::state::AppState;
use crate::utils::cache::{CachedDashboardStats, DashboardCacheKey};
use crate::utils::extractors::{AuthenticatedUser, DbConnection};
use crate::utils::session::SessionManager;
use crate::utils::time::{TimeFormat, human_readable_duration};
use aide::NoApi;
use axum::{
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
    total_heartbeats: i64,
    human_readable_total: String,
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
    NoApi(AuthenticatedUser(user)): NoApi<AuthenticatedUser>,
    NoApi(DbConnection(mut conn)): NoApi<DbConnection>,
    Query(query): Query<DashboardQuery>,
) -> Result<Json<DashboardResponse>, Response> {
    let cookies = cookies.0;

    // get user's session info
    let Some(session_id) = SessionManager::get_session_from_cookies(&cookies) else {
        return Err((StatusCode::UNAUTHORIZED, "Session missing").into_response());
    };

    let Some(session_data) = db_query!(
        SessionManager::validate_session(&app_state.db_pool, session_id).await,
        "Session validation error"
    ) else {
        return Err((StatusCode::UNAUTHORIZED, "Session invalid").into_response());
    };

    let user_timezone = user.timezone.clone();
    let cache_key = DashboardCacheKey {
        user_id: session_data.user_id,
        range: query.range,
        timezone: user_timezone.clone(),
    };

    let (total_heartbeats, dashboard_stats) = match app_state.cache.dashboard.get(&cache_key) {
        Some(cached) => (cached.heartbeat_count, cached.stats),
        None => {
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
        }
    };

    Ok(Json(DashboardResponse {
        total_heartbeats,
        human_readable_total: human_readable_duration(
            dashboard_stats.total_time,
            TimeFormat::NoDays,
        )
        .human_readable,
        range: query.range.as_str().to_string(),
        projects: dashboard_stats.top_projects,
        editors: dashboard_stats.top_editors,
        operating_systems: dashboard_stats.top_oses,
        languages: dashboard_stats.top_languages,
    }))
}
