use chrono::{DateTime, Utc};
use diesel::prelude::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::schema::import_jobs;
use crate::schema::users;
use crate::utils::instrumented;

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

#[derive(Queryable, Serialize, JsonSchema)]
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
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ImportJob {
    pub fn create(conn: &mut PgConnection, user_id: i32) -> QueryResult<ImportJob> {
        let new_job = NewImportJob {
            user_id,
            status: ImportJobStatus::Running.as_str().to_string(),
        };

        instrumented::first("ImportJob::create", || {
            diesel::insert_into(import_jobs::table)
                .values(&new_job)
                .returning(ImportJob::as_returning())
                .get_result(conn)
        })
    }

    pub fn get_latest_for_user(
        conn: &mut PgConnection,
        user_id: i32,
    ) -> QueryResult<Option<ImportJob>> {
        instrumented::first("ImportJob::get_latest_for_user", || {
            import_jobs::table
                .filter(import_jobs::user_id.eq(user_id))
                .order(import_jobs::created_at.desc())
                .first::<ImportJob>(conn)
        })
        .optional()
    }

    pub fn get_active_for_user(
        conn: &mut PgConnection,
        user_id: i32,
    ) -> QueryResult<Option<ImportJob>> {
        instrumented::first("ImportJob::get_active_for_user", || {
            import_jobs::table
                .filter(import_jobs::user_id.eq(user_id))
                .filter(import_jobs::status.eq(ImportJobStatus::Running.as_str()))
                .order(import_jobs::created_at.desc())
                .first::<ImportJob>(conn)
        })
        .optional()
    }

    pub fn get_all_with_users(
        conn: &mut PgConnection,
        limit: i64,
        offset: i64,
    ) -> QueryResult<Vec<ImportJobWithUser>> {
        let results = instrumented::load("ImportJob::get_all_with_users", || {
            import_jobs::table
                .inner_join(
                    crate::schema::users::table
                        .on(crate::schema::users::id.eq(import_jobs::user_id)),
                )
                .select((
                    import_jobs::id,
                    import_jobs::user_id,
                    users::name.nullable(),
                    users::avatar_url.nullable(),
                    import_jobs::status,
                    import_jobs::imported_count,
                    import_jobs::processed_count,
                    import_jobs::request_count,
                    import_jobs::start_date,
                    import_jobs::time_taken,
                    import_jobs::error_message,
                    import_jobs::created_at,
                    import_jobs::updated_at,
                ))
                .order(import_jobs::created_at.desc())
                .limit(limit)
                .offset(offset)
                .load::<ImportJobWithUser>(conn)
        })?;

        Ok(results)
    }

    pub fn count_all(conn: &mut PgConnection) -> QueryResult<i64> {
        instrumented::first("ImportJob::count_all", || {
            import_jobs::table.count().get_result(conn)
        })
    }

    pub fn complete(
        conn: &mut PgConnection,
        id: i64,
        imported_count: i64,
        processed_count: i64,
        request_count: i32,
        start_date: String,
        time_taken: f64,
    ) -> QueryResult<usize> {
        instrumented::execute("ImportJob::complete", || {
            diesel::update(import_jobs::table.find(id))
                .set((
                    import_jobs::status.eq(ImportJobStatus::Completed.as_str()),
                    import_jobs::imported_count.eq(Some(imported_count)),
                    import_jobs::processed_count.eq(Some(processed_count)),
                    import_jobs::request_count.eq(Some(request_count)),
                    import_jobs::start_date.eq(Some(start_date)),
                    import_jobs::time_taken.eq(Some(time_taken)),
                ))
                .execute(conn)
        })
    }

    pub fn fail(conn: &mut PgConnection, id: i64, error_message: &str) -> QueryResult<usize> {
        instrumented::execute("ImportJob::fail", || {
            diesel::update(import_jobs::table.find(id))
                .set((
                    import_jobs::status.eq(ImportJobStatus::Failed.as_str()),
                    import_jobs::error_message.eq(Some(error_message)),
                ))
                .execute(conn)
        })
    }
}
