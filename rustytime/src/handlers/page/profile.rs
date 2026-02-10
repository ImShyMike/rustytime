use crate::db_query;
use crate::models::user::User;
use crate::state::AppState;
use crate::utils::extractors::DbConnection;
use aide::NoApi;
use axum::Json;
use axum::extract::{Path, State};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema, Clone)]
pub struct UserProfileTime {
    pub today: i64,
    pub week: i64,
    pub all_time: i64,
}

#[derive(Serialize, JsonSchema, Clone)]
pub struct UserProfileProject {
    pub name: String,
    pub project_url: Option<String>,
    pub total_seconds: i64,
}

#[derive(Serialize, JsonSchema, Clone)]
pub struct ProfileUser {
    pub username: String,
    pub avatar_url: String,
    pub admin_level: i16,
}

#[derive(Serialize, JsonSchema, Clone)]
pub struct UserProfile {
    pub user: ProfileUser,
    pub projects: Vec<UserProfileProject>,
    pub time: UserProfileTime,
}

pub async fn profile_handler(
    State(app_state): State<AppState>,
    Path(username): Path<String>,
    NoApi(DbConnection(mut conn)): NoApi<DbConnection>,
) -> Result<Json<UserProfile>, Response> {
    let username = username.chars().take(100).collect::<String>();

    // check cache first
    let cache_key = username.clone();
    if let Some(cached) = app_state.cache.profile.get(&cache_key) {
        return Ok(Json(cached));
    }

    // fetch user info and projects from DB
    let user_info = db_query!(
        User::get_user_profile(&mut conn, &username),
        "Failed to fetch user profile"
    );

    // convert to response format
    let response = UserProfile {
        user: ProfileUser {
            username: user_info.user.username,
            avatar_url: user_info.user.avatar_url,
            admin_level: user_info.user.admin_level,
        },
        projects: user_info
            .projects
            .into_iter()
            .map(|p| UserProfileProject {
                name: p.name,
                project_url: p.project_url,
                total_seconds: p.total_seconds,
            })
            .collect(),
        time: UserProfileTime {
            today: user_info.time.today,
            week: user_info.time.week,
            all_time: user_info.time.all_time,
        },
    };

    // store in cache
    app_state.cache.profile.insert(cache_key, response.clone());

    Ok(Json(response))
}
