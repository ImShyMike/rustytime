use std::env;

use crate::db_query;
use crate::models::user::User;
use crate::state::AppState;
use crate::utils::extractors::{AuthenticatedUser, DbConnection};
use crate::utils::session::SessionManager;
use aide::NoApi;
use axum::Json;
use axum::extract::State;
use axum::response::Redirect;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use chrono_tz::Tz;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tower_cookies::Cookies;
use uuid::Uuid;

#[derive(Serialize, JsonSchema)]
pub struct SettingsResponse {
    #[schemars(with = "Option<String>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<Uuid>,
    pub timezone: String,
}

#[derive(Deserialize, JsonSchema)]
pub struct UpdateSettingsRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct UpdateSettingsResponse {
    pub success: bool,
}

/// Handler for the settings page
pub async fn settings_page(
    State(app_state): State<AppState>,
    cookies: NoApi<Cookies>,
    NoApi(AuthenticatedUser(current_user)): NoApi<AuthenticatedUser>,
) -> Result<Json<SettingsResponse>, Response> {
    let cookies = cookies.0;

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

    if show_api_key {
        Ok(Json(SettingsResponse {
            api_key: Some(current_user.api_key),
            timezone: current_user.timezone,
        }))
    } else {
        Ok(Json(SettingsResponse {
            api_key: None,
            timezone: current_user.timezone,
        }))
    }
}

/// Handler for updating user settings
pub async fn update_settings(
    NoApi(AuthenticatedUser(current_user)): NoApi<AuthenticatedUser>,
    NoApi(DbConnection(mut conn)): NoApi<DbConnection>,
    Json(request): Json<UpdateSettingsRequest>,
) -> Result<Json<UpdateSettingsResponse>, Response> {
    // update timezone if provided
    if let Some(ref timezone) = request.timezone {
        if timezone.parse::<Tz>().is_err() {
            return Err((
                StatusCode::BAD_REQUEST,
                "Invalid timezone. Please use a valid IANA timezone like 'America/New_York' or 'Europe/London'.",
            ).into_response());
        }

        User::set_timezone(&mut conn, current_user.id, timezone).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to update settings: {}", e),
            )
                .into_response()
        })?;
    }

    Ok(Json(UpdateSettingsResponse { success: true }))
}
