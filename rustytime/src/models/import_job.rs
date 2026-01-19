use chrono::{DateTime, Utc};
use diesel::prelude::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::db::connection::DbPool;
use crate::schema::import_jobs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ImportJobStatus {
    Running,
    Completed,
    Failed,
}

impl ImportJobStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ImportJobStatus::Running => "running",
            ImportJobStatus::Completed => "completed",
            ImportJobStatus::Failed => "failed",
        }
    }
}

impl From<&str> for ImportJobStatus {
    fn from(s: &str) -> Self {
        match s {
            "running" => ImportJobStatus::Running,
            "completed" => ImportJobStatus::Completed,
            "failed" => ImportJobStatus::Failed,
            _ => ImportJobStatus::Running,
        }
    }
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[diesel(table_name = import_jobs)]
pub struct ImportJob {
    pub id: i64,
    pub user_id: i32,
    pub status: String,
    pub imported_count: Option<i64>,
    pub processed_count: Option<i64>,
    pub request_count: Option<i32>,
    pub start_date: Option<String>,
    pub time_taken: Option<f64>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Debug)]
#[diesel(table_name = import_jobs)]
pub struct NewImportJob {
    pub user_id: i32,
    pub status: String,
}

impl ImportJob {
    pub fn create(pool: &DbPool, user_id: i32) -> QueryResult<ImportJob> {
        let mut conn = pool.get().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        let new_job = NewImportJob {
            user_id,
            status: ImportJobStatus::Running.as_str().to_string(),
        };

        diesel::insert_into(import_jobs::table)
            .values(&new_job)
            .returning(ImportJob::as_returning())
            .get_result(&mut conn)
    }

    pub fn get_latest_for_user(pool: &DbPool, user_id: i32) -> QueryResult<Option<ImportJob>> {
        let mut conn = pool.get().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        import_jobs::table
            .filter(import_jobs::user_id.eq(user_id))
            .order(import_jobs::created_at.desc())
            .first::<ImportJob>(&mut conn)
            .optional()
    }

    pub fn get_active_for_user(pool: &DbPool, user_id: i32) -> QueryResult<Option<ImportJob>> {
        let mut conn = pool.get().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        import_jobs::table
            .filter(import_jobs::user_id.eq(user_id))
            .filter(import_jobs::status.eq(ImportJobStatus::Running.as_str()))
            .order(import_jobs::created_at.desc())
            .first::<ImportJob>(&mut conn)
            .optional()
    }

    pub fn get_all(pool: &DbPool, limit: i64, offset: i64) -> QueryResult<Vec<ImportJob>> {
        let mut conn = pool.get().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        import_jobs::table
            .order(import_jobs::created_at.desc())
            .limit(limit)
            .offset(offset)
            .load::<ImportJob>(&mut conn)
    }

    pub fn count_all(pool: &DbPool) -> QueryResult<i64> {
        let mut conn = pool.get().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        import_jobs::table.count().get_result(&mut conn)
    }

    pub fn complete(
        pool: &DbPool,
        id: i64,
        imported_count: i64,
        processed_count: i64,
        request_count: i32,
        start_date: String,
        time_taken: f64,
    ) -> QueryResult<usize> {
        let mut conn = pool.get().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        diesel::update(import_jobs::table.find(id))
            .set((
                import_jobs::status.eq(ImportJobStatus::Completed.as_str()),
                import_jobs::imported_count.eq(Some(imported_count)),
                import_jobs::processed_count.eq(Some(processed_count)),
                import_jobs::request_count.eq(Some(request_count)),
                import_jobs::start_date.eq(Some(start_date)),
                import_jobs::time_taken.eq(Some(time_taken)),
            ))
            .execute(&mut conn)
    }

    pub fn fail(pool: &DbPool, id: i64, error_message: &str) -> QueryResult<usize> {
        let mut conn = pool.get().map_err(|e| {
            diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::Unknown,
                Box::new(e.to_string()),
            )
        })?;

        diesel::update(import_jobs::table.find(id))
            .set((
                import_jobs::status.eq(ImportJobStatus::Failed.as_str()),
                import_jobs::error_message.eq(Some(error_message)),
            ))
            .execute(&mut conn)
    }
}
