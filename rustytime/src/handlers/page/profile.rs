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
pub struct TimeInfo {
    pub today: i64,
    pub week: i64,
    pub all_time: i64,
}

#[derive(Serialize, JsonSchema, Clone)]
pub struct Project {
    pub name: String,
    pub project_url: Option<String>,
    pub total_seconds: i64,
}

#[derive(Serialize, JsonSchema, Clone)]
pub struct ProfileResponse {
    pub username: String,
    pub avatar_url: String,
    pub projects: Vec<Project>,
    pub time: TimeInfo,
}

pub async fn profile_handler(
    State(app_state): State<AppState>,
    Path(username): Path<String>,
    NoApi(DbConnection(mut conn)): NoApi<DbConnection>,
) -> Result<Json<ProfileResponse>, Response> {
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
    let response = ProfileResponse {
        username: user_info.username,
        avatar_url: user_info.avatar_url,
        projects: user_info
            .projects
            .into_iter()
            .map(|p| Project {
                name: p.name,
                project_url: p.project_url,
                total_seconds: p.total_seconds,
            })
            .collect(),
        time: TimeInfo {
            today: user_info.time.today,
            week: user_info.time.week,
            all_time: user_info.time.all_time,
        },
    };

    // store in cache
    app_state.cache.profile.insert(cache_key, response.clone());

    Ok(Json(response))
}
