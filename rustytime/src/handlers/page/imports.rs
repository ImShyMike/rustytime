use aide::NoApi;
use axum::Json;
use axum::extract::Query;
use axum::{
    Extension, extract::State, http::StatusCode, response::IntoResponse, response::Response,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::db_query;
use crate::models::import_job::ImportJob;
use crate::models::user::User;
use crate::state::AppState;

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
pub struct ImportJobWithUser {
    pub id: i64,
    pub user_id: i32,
    pub user_name: Option<String>,
    pub user_avatar_url: Option<String>,
    pub status: String,
    pub imported_count: Option<i64>,
    pub processed_count: Option<i64>,
    pub request_count: Option<i32>,
    pub start_date: Option<String>,
    pub time_taken: Option<f64>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, JsonSchema)]
pub struct AdminImportsResponse {
    pub imports: Vec<ImportJobWithUser>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}

pub async fn admin_imports(
    State(app_state): State<AppState>,
    Query(query): Query<ImportsQuery>,
    user: NoApi<Option<Extension<User>>>,
) -> Result<Json<AdminImportsResponse>, Response> {
    let current_user = user
        .0
        .expect("User should be authenticated since middleware validated authentication")
        .0;

    if !current_user.is_admin() {
        return Err((StatusCode::FORBIDDEN, "No permission").into_response());
    }

    let limit = query.limit.clamp(1, 100);
    let offset = query.offset.max(0);

    let total = db_query!(
        ImportJob::count_all(&app_state.db_pool),
        "Failed to count import jobs"
    );

    let jobs = db_query!(
        ImportJob::get_all(&app_state.db_pool, limit, offset),
        "Failed to fetch import jobs"
    );

    let user_ids: Vec<i32> = jobs.iter().map(|j| j.user_id).collect();
    let users = db_query!(
        User::get_by_ids(&app_state.db_pool, &user_ids),
        "Failed to fetch users"
    );

    let imports: Vec<ImportJobWithUser> = jobs
        .into_iter()
        .map(|job| {
            let user = users.iter().find(|u| u.id == job.user_id);
            ImportJobWithUser {
                id: job.id,
                user_id: job.user_id,
                user_name: user.map(|u| u.name.clone()),
                user_avatar_url: user.map(|u| u.avatar_url.clone()),
                status: job.status,
                imported_count: job.imported_count,
                processed_count: job.processed_count,
                request_count: job.request_count,
                start_date: job.start_date,
                time_taken: job.time_taken,
                error_message: job.error_message,
                created_at: job.created_at.to_rfc3339(),
                updated_at: job.updated_at.to_rfc3339(),
            }
        })
        .collect();

    Ok(Json(AdminImportsResponse {
        imports,
        total,
        limit,
        offset,
    }))
}
