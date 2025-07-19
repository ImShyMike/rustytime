use axum::{
    extract::State,
    response::Html,
    http::StatusCode,
};
use tower_cookies::Cookies;
use crate::state::AppState;
use crate::utils::session::SessionManager;

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

    Ok(Html(format!(
        r#"
        <html>
            <head>
                <title>rustytime Dashboard</title>
                <style>
                    body {{ font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; }}
                    .user-info {{ background: #f5f5f5; padding: 20px; border-radius: 8px; margin: 20px 0; }}
                    .avatar {{ border-radius: 50%; }}
                    .api-key {{ background: #e8f4f8; padding: 10px; border-radius: 4px; font-family: monospace; }}
                    .success {{ color: #28a745; }}
                    .logout {{ margin-top: 20px; }}
                    .logout a {{ background: #dc3545; color: white; padding: 10px 20px; text-decoration: none; border-radius: 4px; }}
                </style>
            </head>
            <body>
                <h1 class="success">🎯 rustytime Dashboard</h1>
                
                <div class="user-info">
                    <img src="{}" alt="Avatar" width="100" height="100" class="avatar">
                    <h3>User Information</h3>
                    <p><strong>Name:</strong> {}</p>
                    <p><strong>GitHub ID:</strong> {}</p>
                    <p><strong>Account Created:</strong> {}</p>
                    <p><strong>Session Expires:</strong> {}</p>
                </div>

                <div class="api-key">
                    <h3>Your API Key</h3>
                    <p><strong>API Key:</strong> {}</p>
                    <p><small>Use this key to authenticate with the API</small></p>
                </div>

                <div class="logout">
                    <a href="/auth/github/logout">Logout</a>
                </div>
            </body>
        </html>
        "#,
        user.avatar_url.as_deref().unwrap_or(""),
        user.name.as_deref().unwrap_or("Unknown"),
        session_data.github_user_id,
        user.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
        session_data.expires_at.format("%Y-%m-%d %H:%M:%S UTC"),
        user.api_key
    )))
}