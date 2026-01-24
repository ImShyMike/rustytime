use aide::NoApi;
use axum::Json;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use tower_cookies::Cookies;
use tracing::{error, info};

use crate::db_query;
use crate::jobs::import::enqueue_import;
use crate::models::import_job::{ImportJob, ImportJobStatus};
use crate::state::AppState;
use crate::utils::extractors::AuthenticatedUser;
use crate::utils::extractors::DbConnection;
use crate::utils::session::SessionManager;

#[derive(Deserialize, JsonSchema)]
pub struct ImportQuery {
    api_key: String,
}

#[derive(Serialize, JsonSchema)]
pub struct ImportStartResponse {
    job_id: i64,
    status: String,
    message: String,
}

#[derive(Serialize, JsonSchema)]
pub struct ImportStatusResponse {
    job_id: i64,
    status: String,
    imported_count: Option<i64>,
    processed_count: Option<i64>,
    request_count: Option<i32>,
    start_date: Option<String>,
    time_taken: Option<f64>,
    error_message: Option<String>,
    created_at: String,
    updated_at: String,
}

impl From<ImportJob> for ImportStatusResponse {
    fn from(job: ImportJob) -> Self {
        Self {
            job_id: job.id,
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
    }
}

pub async fn import_heartbeats(
    State(app_state): State<AppState>,
    Query(query): Query<ImportQuery>,
    cookies: NoApi<Cookies>,
    NoApi(AuthenticatedUser(current_user)): NoApi<AuthenticatedUser>,
    NoApi(DbConnection(mut conn)): NoApi<DbConnection>,
) -> Result<Json<ImportStartResponse>, Response> {
    let Some(session_id) = SessionManager::get_session_from_cookies(&cookies) else {
        return Err((StatusCode::UNAUTHORIZED, "User session is invalid").into_response());
    };

    let Some(session_data) = db_query!(
        SessionManager::validate_session(&app_state.db_pool, session_id).await,
        "Session validation error"
    ) else {
        return Err((StatusCode::UNAUTHORIZED, "User session is invalid").into_response());
    };

    if session_data.impersonated_by.is_some() && !current_user.is_owner() {
        return Err((
            StatusCode::FORBIDDEN,
            "Impersonators cannot perform data imports",
        )
            .into_response());
    }

    let api_key = query.api_key.trim().to_string();
    if api_key.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "api_key query parameter is required",
        )
            .into_response());
    }

    let user_id = current_user.id;

    if let Ok(Some(active_job)) = ImportJob::get_active_for_user(&mut conn, user_id) {
        return Err((
            StatusCode::CONFLICT,
            format!(
                "An import job is already {} for this user (job_id: {})",
                active_job.status, active_job.id
            ),
        )
            .into_response());
    }

    let import_job = db_query!(
        ImportJob::create(&mut conn, user_id),
        "Failed to create import job"
    );

    let import_store = app_state.import_store.read().await;
    let Some(ref store) = *import_store else {
        error!("Import store not initialized");
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            "Import service is not available",
        )
            .into_response());
    };

    if let Err(e) = enqueue_import(store, user_id, api_key, import_job.id).await {
        error!(error = ?e, job_id = import_job.id, "Failed to enqueue import job");
        let _ = ImportJob::fail(&mut conn, import_job.id, "Failed to enqueue job");
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to start import job",
        )
            .into_response());
    }

    info!(
        user_id = user_id,
        job_id = import_job.id,
        "Import job enqueued"
    );

    Ok(Json(ImportStartResponse {
        job_id: import_job.id,
        status: ImportJobStatus::Running.as_str().to_string(),
        message: "Import job started".to_string(),
    }))
}

pub async fn import_status(
    NoApi(AuthenticatedUser(current_user)): NoApi<AuthenticatedUser>,
    NoApi(DbConnection(mut conn)): NoApi<DbConnection>,
) -> Result<Json<ImportStatusResponse>, Response> {
    let user_id = current_user.id;

    let job = db_query!(
        ImportJob::get_latest_for_user(&mut conn, user_id),
        "Failed to get import job"
    );

    match job {
        Some(j) => Ok(Json(ImportStatusResponse::from(j))),
        None => Err((StatusCode::NOT_FOUND, "No import jobs found").into_response()),
    }
}
