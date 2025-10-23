use std::env;

use axum::Json;
use axum::extract::Path;
use axum::response::Redirect;
use axum::{
    Extension,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tower_cookies::Cookies;

use crate::models::heartbeat::{Heartbeat, LanguageCount, ProjectCount};
use crate::models::session::Session;
use crate::models::user::{PartialUser, User};
use crate::state::AppState;
use crate::utils::session::{ImpersonationContext, SessionManager};
use crate::{db_query, get_db_conn};

#[derive(Serialize)]
pub struct FormattedDailyActivity {
    pub date: String,
    pub count: i64,
}

#[derive(Serialize)]
pub struct AdminDashboardResponse {
    pub total_users: i64,
    pub total_heartbeats: i64,
    pub heartbeats_last_hour: i64,
    pub heartbeats_last_24h: i64,
    pub requests_per_second: f64,
    pub top_languages: Vec<LanguageCount>,
    pub top_projects: Vec<ProjectCount>,
    pub daily_activity: Vec<FormattedDailyActivity>,
    pub all_users: Vec<PartialUser>,
}

#[derive(Serialize)]
pub struct ImpersonationStatus {
    pub admin_id: i32,
    pub admin_name: String,
    pub admin_avatar_url: String,
    pub target_id: i32,
    pub target_name: String,
    pub target_avatar_url: String,
}

pub async fn admin_dashboard(
    State(app_state): State<AppState>,
    user: Option<Extension<User>>,
) -> Result<Json<AdminDashboardResponse>, Response> {
    // check if user is an admin
    let current_user = user
        .expect("User should be authenticated since middleware validated authentication")
        .0;

    if !current_user.is_admin() {
        return Err((StatusCode::FORBIDDEN, "No permission").into_response());
    }

    // get database connection
    let mut conn = get_db_conn!(app_state);

    // fetch raw data
    let raw_daily_activity = db_query!(
        Heartbeat::get_daily_activity_last_week(&mut conn),
        "Failed to fetch daily activity"
    );
    let all_users = db_query!(User::list_all_users(&mut conn), "Failed to fetch users");
    let include_api_key = current_user.is_owner();

    let partial_users = all_users
        .iter()
        .map(|user| PartialUser {
            id: user.id,
            github_id: user.github_id,
            name: user.name.clone(),
            avatar_url: user.avatar_url.clone(),
            admin_level: user.admin_level,
            is_banned: user.is_banned,
            api_key: include_api_key.then_some(user.api_key.clone()),
            created_at: user.created_at,
            updated_at: user.updated_at,
        })
        .collect();

    // convert to formatted versions
    let daily_activity: Vec<FormattedDailyActivity> = raw_daily_activity
        .into_iter()
        .map(|activity| FormattedDailyActivity {
            date: activity.date.format("%m-%d").to_string(),
            count: activity.count,
        })
        .collect();

    // fetch all statistics and return response
    Ok(Json(AdminDashboardResponse {
        total_users: db_query!(User::count_total_users(&mut conn, false)),
        total_heartbeats: db_query!(Heartbeat::count_total_heartbeats(&mut conn)),
        heartbeats_last_hour: db_query!(Heartbeat::count_heartbeats_last_hour(&mut conn)),
        heartbeats_last_24h: db_query!(Heartbeat::count_heartbeats_last_24h(&mut conn)),
        requests_per_second: (app_state.metrics.get_metrics().requests_per_second * 1000.0).round()
            / 1000.0,
        top_languages: db_query!(Heartbeat::get_top_languages(&mut conn, 10)),
        top_projects: db_query!(Heartbeat::get_top_projects(&mut conn, 10)),
        daily_activity,
        all_users: partial_users,
    }))
}

pub async fn impersonate_user(
    State(app_state): State<AppState>,
    Path(user_id): Path<i64>,
    cookies: Cookies,
    impersonation: Option<Extension<ImpersonationContext>>,
    user: Option<Extension<User>>,
) -> Result<Redirect, Response> {
    let session_user = user
        .expect("User should be authenticated since middleware validated authentication")
        .0;

    let frontend_url =
        env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());

    let Some(session_id) = SessionManager::get_session_from_cookies(&cookies) else {
        return Ok(Redirect::to(&format!(
            "{}/?error=session_missing",
            frontend_url
        )));
    };

    let mut conn = get_db_conn!(app_state);

    let Some(session_data) = db_query!(
        SessionManager::validate_session(&app_state.db_pool, session_id).await,
        "Failed to validate session"
    ) else {
        return Ok(Redirect::to(&format!(
            "{}/?error=session_invalid",
            frontend_url
        )));
    };

    let acting_admin = if let Some(ctx) = impersonation.as_ref() {
        ctx.0.admin.clone()
    } else if let Some(admin_id) = session_data.impersonated_by {
        let Some(admin) = db_query!(
            User::get_by_id(&mut conn, admin_id),
            "Failed to fetch impersonating admin"
        ) else {
            return Err((StatusCode::FORBIDDEN, "Impersonation source not found").into_response());
        };
        admin
    } else {
        session_user.clone()
    };

    if session_data
        .impersonated_by
        .map(|admin_id| admin_id != acting_admin.id)
        .unwrap_or(false)
    {
        return Err((StatusCode::FORBIDDEN, "Impersonation mismatch").into_response());
    }

    if !acting_admin.is_admin() {
        return Err((StatusCode::FORBIDDEN, "No permission").into_response());
    }

    let Some(target_user) = db_query!(
        User::get_by_id(&mut conn, user_id as i32),
        "Failed to fetch target user"
    ) else {
        return Err((StatusCode::NOT_FOUND, "User not found").into_response());
    };

    if acting_admin.admin_level <= target_user.admin_level && acting_admin.id != target_user.id {
        return Err((StatusCode::BAD_REQUEST, "Cannot impersonate another admin").into_response());
    }

    let updated_session = if target_user.id == acting_admin.id {
        db_query!(
            Session::clear_impersonation(&mut conn, session_id, &acting_admin),
            "Failed to clear impersonation"
        )
    } else {
        db_query!(
            Session::impersonate(&mut conn, session_id, &target_user, acting_admin.id),
            "Failed to impersonate user"
        )
    };

    let session_cookie = SessionManager::create_session_cookie(updated_session.id);
    cookies.add(session_cookie);

    if target_user.id == acting_admin.id {
        Ok(Redirect::to(&format!(
            "{}/?impersonation=cleared",
            frontend_url
        )))
    } else {
        Ok(Redirect::to(&format!(
            "{}/?impersonation=active&user={}",
            frontend_url, target_user.id
        )))
    }
}
