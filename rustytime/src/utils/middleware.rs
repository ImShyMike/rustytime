use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{IntoResponse, Response, Redirect},
};
use tower_cookies::Cookies;

use crate::state::AppState;
use crate::utils::session::SessionManager;

/// middleware to require authentication
pub async fn require_auth(
    State(app_state): State<AppState>,
    cookies: Cookies,
    request: Request,
    next: Next,
) -> Response {
    if !SessionManager::is_authenticated(&cookies, &app_state.db_pool).await {
        return Redirect::to("/auth/github/login").into_response();
    }

    next.run(request).await
}

/// middleware to optionally check authentication (doesn't fail if not authenticated)
pub async fn optional_auth(
    State(app_state): State<AppState>,
    cookies: Cookies,
    mut request: Request,
    next: Next,
) -> Response {
    // try to get current user and add to request extensions
    if let Ok(Some(user)) = SessionManager::get_current_user(&cookies, &app_state.db_pool).await {
        request.extensions_mut().insert(user);
    }

    next.run(request).await
}
