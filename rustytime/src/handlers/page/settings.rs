use std::env;

use crate::db_query;
use crate::models::user::User;
use crate::state::AppState;
use crate::utils::session::SessionManager;
use axum::Json;
use axum::extract::State;
use axum::response::Redirect;
use axum::{
    Extension,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tower_cookies::Cookies;
use uuid::Uuid;

#[derive(Serialize)]
pub struct SettingsResponse {
    pub api_key: Option<Uuid>,
}

/// Handler for the settings page
pub async fn settings_page(
    State(app_state): State<AppState>,
    cookies: Cookies,
    user: Option<Extension<User>>,
) -> Result<Json<SettingsResponse>, Response> {
    // get current user
    let current_user = user
        .expect("User should be authenticated since middleware validated authentication")
        .0;

    let frontend_url =
        env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());

    let Some(session_id) = SessionManager::get_session_from_cookies(&cookies) else {
        return Err(
            Redirect::to(&format!("{}/?error=session_missing", frontend_url)).into_response(),
        );
    };

    let Some(session_data) = db_query!(
        SessionManager::validate_session(&app_state.db_pool, session_id).await,
        "Session validation error"
    ) else {
        return Err(
            Redirect::to(&format!("{}/?error=session_invalid", frontend_url)).into_response(),
        );
    };

    let show_api_key = session_data.impersonated_by.is_none() || current_user.is_owner();

    Ok(Json(SettingsResponse {
        api_key: if show_api_key {
            Some(current_user.api_key)
        } else {
            None
        },
    }))
}
