use axum::{
    extract::{Request, State},
    http::{HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use reqwest::Method;
use tower_cookies::Cookies;

use crate::state::AppState;
use crate::utils::session::{ImpersonationContext, SessionManager};
use tower_http::cors::CorsLayer;

/// Middleware to require authentication
pub async fn require_auth(
    State(app_state): State<AppState>,
    cookies: Cookies,
    mut request: Request,
    next: Next,
) -> Response {
    match SessionManager::resolve_session(&cookies, &app_state.db_pool).await {
        Ok(Some(resolved)) => {
            {
                let extensions = request.extensions_mut();
                extensions.insert(resolved.user.clone());
                if let Some(admin) = resolved.impersonator.clone() {
                    extensions.insert(ImpersonationContext { admin });
                }
            }
            next.run(request).await
        }
        Ok(None) => (StatusCode::UNAUTHORIZED, "Authentication required").into_response(),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response(),
    }
}

// Middleware to inject the user if authenticated
// pub async fn optional_auth(
//     State(app_state): State<AppState>,
//     cookies: Cookies,
//     mut request: Request,
//     next: Next,
// ) -> Response {
//     // try to get current user and add to request extensions
//     if let Ok(Some(user)) = SessionManager::get_current_user(&cookies, &app_state.db_pool).await {
//         request.extensions_mut().insert(user);
//     }

//     next.run(request).await
// }

/// Middleware to require admin privileges
pub async fn require_admin(
    State(app_state): State<AppState>,
    cookies: Cookies,
    mut request: Request,
    next: Next,
) -> Response {
    match SessionManager::resolve_session(&cookies, &app_state.db_pool).await {
        Ok(Some(resolved)) => {
            let is_admin = resolved.user.is_admin()
                || resolved
                    .impersonator
                    .as_ref()
                    .map(|admin| admin.is_admin())
                    .unwrap_or(false);

            if !is_admin {
                return (StatusCode::FORBIDDEN, "Admin access required").into_response();
            }

            {
                let extensions = request.extensions_mut();
                extensions.insert(resolved.user.clone());
                if let Some(admin) = resolved.impersonator.clone() {
                    extensions.insert(ImpersonationContext { admin });
                }
            }

            next.run(request).await
        }
        Ok(None) => {
            // user is not authenticated
            (StatusCode::UNAUTHORIZED, "Authentication required").into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response(),
    }
}

/// Middleware to track request metrics
#[inline(always)]
pub async fn track_metrics(
    State(app_state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    // record the request metrics
    app_state.metrics.record_request();

    next.run(request).await
}

/// Layer to add CORS
pub fn cors_layer() -> CorsLayer {
    let mut allowed_origins = vec![
        "http://localhost:5173".parse::<HeaderValue>().unwrap(), // SvelteKit dev
        "http://localhost:4173".parse::<HeaderValue>().unwrap(), // SvelteKit preview
        "http://localhost:8787".parse::<HeaderValue>().unwrap(), // Local Cloudflare Workers dev
    ];

    // add production domain from environment variable
    if let Ok(prod_origin) = std::env::var("FRONTEND_URL") {
        if let Ok(origin) = prod_origin.parse::<HeaderValue>() {
            allowed_origins.push(origin);
        }
    }

    CorsLayer::new()
        .allow_origin(allowed_origins)
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
            axum::http::header::COOKIE,
            axum::http::header::SET_COOKIE,
            axum::http::header::ORIGIN,
            axum::http::header::REFERER,
            axum::http::header::USER_AGENT,
        ])
        .expose_headers([
            axum::http::header::SET_COOKIE,
            axum::http::header::CONTENT_TYPE,
        ])
        .allow_credentials(true)
}
