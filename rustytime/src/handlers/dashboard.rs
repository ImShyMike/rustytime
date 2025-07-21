use crate::state::AppState;
use crate::utils::session::SessionManager;
use axum::{extract::State, http::StatusCode, response::Html};
use tower_cookies::Cookies;

/// Handler for the dashboard page (will likely be done using svelteKit later)
pub async fn dashboard(
    State(app_state): State<AppState>,
    cookies: Cookies,
) -> Result<Html<String>, StatusCode> {
    // get current user from session - middleware guarantees user is authenticated
    let user = SessionManager::get_current_user(&cookies, &app_state.db_pool)
        .await
        .map_err(|err| {
            eprintln!("Database error: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .expect("User should be authenticated by middleware");

    // get user's session info
    let session_id = SessionManager::get_session_from_cookies(&cookies)
        .expect("Session should exist since middleware validated authentication");

    let session_data = SessionManager::validate_session(&app_state.db_pool, session_id)
        .await
        .map_err(|err| {
            eprintln!("Session validation error: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .expect("Session should be valid since middleware validated authentication");

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
                api_key => user.api_key.to_string()
            },
        )
        .map_err(|err| {
            eprintln!("Template rendering error: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Html(rendered))
}
