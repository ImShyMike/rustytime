use crate::models::project::Project as ProjectModel;
use crate::models::user::User;
use crate::state::AppState;
use crate::{db_query, get_db_conn};
use aide::NoApi;
use axum::Json;
use axum::extract::Path;
use axum::{
    Extension,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, JsonSchema)]
pub struct SimpleProject {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize, JsonSchema)]
pub struct ProjectsListResponse {
    pub projects: Vec<SimpleProject>,
}

#[derive(Deserialize, JsonSchema)]
pub struct RepoUrlRequest {
    pub repo_url: Option<String>,
}

/// Handler for the projects list
pub async fn projects_list(
    State(app_state): State<AppState>,
    user: NoApi<Option<Extension<User>>>,
) -> Result<Json<ProjectsListResponse>, Response> {
    // get current user
    let current_user = user
        .0
        .expect("User should be authenticated since middleware validated authentication")
        .0;

    // get database connection
    let mut conn = get_db_conn!(app_state);

    // get all projects
    let projects = db_query!(
        ProjectModel::list_user_projects(&mut conn, current_user.id),
        "Failed to fetch projects"
    )
    .iter()
    .map(|proj| SimpleProject {
        id: proj.id,
        name: proj.name.clone(),
    })
    .collect();

    Ok(Json(ProjectsListResponse { projects }))
}

pub async fn set_project_repo(
    State(app_state): State<AppState>,
    user: NoApi<Option<Extension<User>>>,
    Path(project_id): Path<i32>,
    repo_url: Json<RepoUrlRequest>,
) -> Result<Response, Response> {
    // get current user
    let current_user = user
        .0
        .expect("User should be authenticated since middleware validated authentication")
        .0;

    // get database connection
    let mut conn = get_db_conn!(app_state);

    // validate the url if provided
    if let Some(url) = &repo_url.repo_url {
        let parsed = url::Url::parse(url).map_err(|_| {
            (
                StatusCode::BAD_REQUEST,
                "Invalid repository URL".to_string(),
            )
                .into_response()
        })?;

        if parsed.scheme() != "http" && parsed.scheme() != "https" {
            return Err((
                StatusCode::BAD_REQUEST,
                "Repository URL must use http or https".to_string(),
            )
                .into_response());
        }

        if url.len() > 128 {
            return Err((
                StatusCode::BAD_REQUEST,
                "Repository URL is too long".to_string(),
            )
                .into_response());
        }
    }

    // set project repo url
    db_query!(
        ProjectModel::set_repo_url(&mut conn, project_id, current_user.id, &repo_url.repo_url),
        "Failed to set project repo URL"
    );

    Ok(StatusCode::OK.into_response())
}
