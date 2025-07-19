use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use tower_cookies::Cookies;

use crate::db::DbPool;
use crate::utils::session::SessionManager;

/// middleware to require authentication
pub async fn require_auth(
    State(pool): State<DbPool>,
    cookies: Cookies,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if !SessionManager::is_authenticated(&cookies, &pool).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(request).await)
}

/// middleware to optionally check authentication (doesn't fail if not authenticated)
pub async fn optional_auth(
    State(pool): State<DbPool>,
    cookies: Cookies,
    mut request: Request,
    next: Next,
) -> Response {
    // try to get current user and add to request extensions
    if let Ok(Some(user)) = SessionManager::get_current_user(&cookies, &pool).await {
        request.extensions_mut().insert(user);
    }

    next.run(request).await
}
