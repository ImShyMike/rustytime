use crate::db_query;
use crate::models::project::Project as ProjectModel;
use crate::state::AppState;
use crate::utils::extractors::{AuthenticatedUser, DbConnection};
use aide::NoApi;
use axum::Json;
use axum::extract::{Path, State};
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const MAX_PROJECT_URL_LENGTH: usize = 255;

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
pub struct UpdateProjectRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hidden: Option<bool>,
}

/// Handler for the projects list
pub async fn projects_list(
    NoApi(AuthenticatedUser(current_user)): NoApi<AuthenticatedUser>,
    NoApi(DbConnection(mut conn)): NoApi<DbConnection>,
) -> Result<Json<ProjectsListResponse>, Response> {
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

pub async fn update_project(
    State(state): State<AppState>,
    NoApi(AuthenticatedUser(current_user)): NoApi<AuthenticatedUser>,
    NoApi(DbConnection(mut conn)): NoApi<DbConnection>,
    Path(project_id): Path<i32>,
    Json(request): Json<UpdateProjectRequest>,
) -> Result<Response, Response> {
    let mut cache_hidden: Option<bool> = None;
    let mut cache_project_url: Option<Option<String>> = None;

    // validate and set project_url if provided
    if let Some(ref url) = request.project_url {
        if !url.is_empty() {
            let parsed = url::Url::parse(url).map_err(|_| {
                (StatusCode::BAD_REQUEST, "Invalid project URL".to_string()).into_response()
            })?;

            if parsed.scheme() != "http" && parsed.scheme() != "https" {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Project URL must use http or https".to_string(),
                )
                    .into_response());
            }

            if url.len() > MAX_PROJECT_URL_LENGTH {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Project URL is too long".to_string(),
                )
                    .into_response());
            }
        }

        // convert empty string to None
        let project_url_value = if url.is_empty() {
            None
        } else {
            Some(url.clone())
        };
        db_query!(
            ProjectModel::set_project_url(
                &mut conn,
                project_id,
                current_user.id,
                &project_url_value
            ),
            "Failed to set project URL"
        );

        cache_project_url = Some(project_url_value);
    }

    // set hidden if provided
    if let Some(hidden) = request.hidden {
        db_query!(
            ProjectModel::set_hidden(&mut conn, project_id, current_user.id, hidden),
            "Failed to set project hidden status"
        );

        cache_hidden = Some(hidden);
    }

    if cache_hidden.is_some() || cache_project_url.is_some() {
        state.cache.update_project_settings(
            current_user.id,
            project_id,
            cache_hidden,
            cache_project_url,
        );
    }

    Ok(StatusCode::OK.into_response())
}
