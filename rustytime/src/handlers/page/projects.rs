use crate::db_query;
use crate::models::project::Project as ProjectModel;
use crate::utils::extractors::{AuthenticatedUser, DbConnection};
use crate::utils::time::{TimeFormat, human_readable_duration};
use aide::NoApi;
use axum::Json;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Serialize, JsonSchema)]
pub struct Project {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    pub repo_url: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub total_seconds: i64,
    pub human_readable_total: String,
}

#[derive(Serialize, JsonSchema)]
pub struct ProjectsDashboardResponse {
    pub projects: Vec<Project>,
}

/// Handler for the projects dashboard page
pub async fn projects_dashboard(
    NoApi(AuthenticatedUser(current_user)): NoApi<AuthenticatedUser>,
    NoApi(DbConnection(mut conn)): NoApi<DbConnection>,
) -> Result<Json<ProjectsDashboardResponse>, Response> {
    // get projects with total time
    let project_rows = db_query!(
        ProjectModel::list_projects_by_user_with_time(&mut conn, current_user.id),
        "Failed to fetch projects"
    );

    // map to response format
    let projects: Vec<Project> = project_rows
        .into_iter()
        .map(|(proj, time)| Project {
            id: proj.id,
            user_id: proj.user_id,
            name: proj.name,
            repo_url: proj.repo_url,
            created_at: proj.created_at,
            updated_at: proj.updated_at,
            total_seconds: time,
            human_readable_total: human_readable_duration(time, TimeFormat::NoDays).human_readable,
        })
        .collect();

    Ok(Json(ProjectsDashboardResponse { projects }))
}
