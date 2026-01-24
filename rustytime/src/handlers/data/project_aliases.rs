use crate::models::project_alias::{NewProjectAlias, ProjectAlias as ProjectAliasModel};
use crate::state::AppState;
use crate::utils::auth::AuthenticatedUser;
use crate::{db_query, get_db_conn};
use aide::NoApi;
use axum::Json;
use axum::extract::Path;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use schemars::JsonSchema;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, JsonSchema)]
pub struct AliasRecord {
    pub id: i32,
    pub project_id: i32,
}

#[derive(Serialize, JsonSchema)]
pub struct ParsedProjectAlias {
    pub project_id: i32,
    pub aliases: Vec<AliasRecord>,
}

#[derive(Serialize, JsonSchema)]
pub struct ProjectAliasesResponse {
    pub aliases: Vec<ParsedProjectAlias>,
}

/// Handler for the project aliases
pub async fn project_aliases(
    State(app_state): State<AppState>,
    NoApi(AuthenticatedUser(current_user)): NoApi<AuthenticatedUser>,
) -> Result<Json<ProjectAliasesResponse>, Response> {

    // get database connection
    let mut conn = get_db_conn!(app_state);

    // get all project aliases
    let alias_records = db_query!(
        ProjectAliasModel::list_user_project_aliases(&mut conn, current_user.id),
        "Failed to fetch project aliases"
    );

    let mut grouped_aliases: HashMap<i32, Vec<AliasRecord>> = HashMap::new();
    for alias in alias_records {
        grouped_aliases
            .entry(alias.alias_to)
            .or_default()
            .push(AliasRecord {
                id: alias.id,
                project_id: alias.project_id,
            });
    }

    let response_aliases = grouped_aliases
        .into_iter()
        .map(|(project_id, aliases)| ParsedProjectAlias {
            project_id,
            aliases,
        })
        .collect();

    Ok(Json(ProjectAliasesResponse {
        aliases: response_aliases,
    }))
}

pub async fn add_project_alias(
    State(app_state): State<AppState>,
    NoApi(AuthenticatedUser(current_user)): NoApi<AuthenticatedUser>,
    Path((id, alias_id)): Path<(i32, i32)>,
) -> Result<StatusCode, Response> {

    let mut conn = get_db_conn!(app_state);

    let new_alias = NewProjectAlias {
        user_id: current_user.id,
        project_id: alias_id,
        alias_to: id,
    };

    db_query!(
        ProjectAliasModel::create(&mut conn, &new_alias),
        "Failed to create project alias"
    );

    Ok(StatusCode::CREATED)
}

pub async fn delete_project_alias(
    State(app_state): State<AppState>,
    NoApi(AuthenticatedUser(current_user)): NoApi<AuthenticatedUser>,
    Path(id): Path<i32>,
) -> Result<StatusCode, Response> {

    let mut conn = get_db_conn!(app_state);

    db_query!(
        ProjectAliasModel::delete_project_alias(&mut conn, current_user.id, id),
        "Failed to delete project alias"
    );

    Ok(StatusCode::OK)
}
