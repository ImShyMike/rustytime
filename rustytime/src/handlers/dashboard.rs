use crate::models::heartbeat::Heartbeat;
use crate::models::user::User;
use crate::state::AppState;
use crate::utils::session::SessionManager;
use crate::utils::time::human_readable_duration;
use axum::{
    Extension,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use tower_cookies::Cookies;

/// Handler for the dashboard page (will likely be done using svelteKit later)
pub async fn dashboard(
    State(app_state): State<AppState>,
    cookies: Cookies,
    user: Option<Extension<User>>,
) -> Result<Html<String>, Response> {
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

    let session_data = SessionManager::validate_session(&app_state.db_pool, session_id)
        .await
        .map_err(|err| {
            eprintln!("Session validation error: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
        })?
        .expect("Session should be valid since middleware validated authentication");

    let mut conn = app_state.db_pool.get().map_err(|err| {
        eprintln!("Database connection error: {}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
    })?;

    // get total duration
    let total_seconds = Heartbeat::get_user_total_duration_seconds(&mut conn, session_data.user_id)
        .map_err(|err| {
            eprintln!("Database error fetching heartbeats: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
        })?;

    // get heartbeat count
    let total_heartbeats = Heartbeat::get_user_heartbeat_count(&mut conn, session_data.user_id)
        .map_err(|err| {
            eprintln!("Database error getting heartbeat count: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
        })? as usize;

    let rendered = app_state
        .template_engine
        .render(
            "dashboard.html",
            minijinja::context! {
                avatar_url => user.avatar_url.as_deref().unwrap_or(""),
                user_name => user.name.as_deref().unwrap_or("Unknown"),
                github_id => session_data.github_user_id,
                created_at => user.created_at.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                expires_at => session_data.expires_at.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                api_key => user.api_key.to_string(),
                total_heartbeats => total_heartbeats,
                formatted_time => human_readable_duration(total_seconds).human_readable,
            },
        )
        .map_err(|err| {
            eprintln!("Template rendering error: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
        })?;

    Ok(Html(rendered))
}
