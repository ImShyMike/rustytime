use aide::NoApi;
use axum::Json;
use axum::extract::Query;
use axum::{http::StatusCode, response::IntoResponse, response::Response};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::db_query;
use crate::models::import_job::ImportJob;
use crate::models::import_job::ImportJobWithUser;
use crate::utils::extractors::{AuthenticatedUser, DbConnection};

#[derive(Deserialize, JsonSchema)]
pub struct ImportsQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Serialize, JsonSchema)]
pub struct AdminImportsResponse {
    pub imports: Vec<ImportJobWithUser>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

pub async fn admin_imports(
    Query(query): Query<ImportsQuery>,
    NoApi(AuthenticatedUser(current_user)): NoApi<AuthenticatedUser>,
    NoApi(DbConnection(mut conn)): NoApi<DbConnection>,
) -> Result<Json<AdminImportsResponse>, Response> {
    if !current_user.is_owner() {
        return Err((StatusCode::FORBIDDEN, "No permission").into_response());
    }

    let limit = query.limit.clamp(1, 100);
    let offset = query.offset.max(0);

    let total = db_query!(
        ImportJob::count_all(&mut conn),
        "Failed to count import jobs"
    );

    let imports = db_query!(
        ImportJob::get_all_with_users(&mut conn, limit, offset),
        "Failed to fetch import jobs"
    );

    Ok(Json(AdminImportsResponse {
        imports,
        total,
        limit,
        offset,
    }))
}
