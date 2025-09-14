use axum::http::HeaderMap;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Date, Nullable, Text};
use ipnetwork::IpNetwork;
use lang_types::Language;
use serde::{Deserialize, Serialize};

use crate::schema::heartbeats;
use crate::utils::http::parse_user_agent;

const TIMEOUT_SECONDS: i32 = 120; // 2 minutes in seconds

// Character limits
const MAX_ENTITY_LENGTH: usize = 512;
const MAX_TYPE_LENGTH: usize = 50;
const MAX_PROJECT_LENGTH: usize = 100;
const MAX_BRANCH_LENGTH: usize = 100;
const MAX_LANGUAGE_LENGTH: usize = 50;
const MAX_CATEGORY_LENGTH: usize = 50;
const MAX_EDITOR_LENGTH: usize = 50;
const MAX_OS_LENGTH: usize = 100;
const MAX_MACHINE_LENGTH: usize = 100;
const MAX_USER_AGENT_LENGTH: usize = 255;

/// Truncate a string to the specified maximum length, respecting UTF-8 boundaries
#[inline(always)]
fn truncate_string(s: String, max_length: usize) -> String {
    if s.chars().count() <= max_length {
        s
    } else {
        s.chars().take(max_length).collect()
    }
}

/// Truncate an optional string to the specified maximum length
#[inline(always)]
fn truncate_optional_string(s: Option<String>, max_length: usize) -> Option<String> {
    s.map(|s| truncate_string(s, max_length))
}

#[derive(QueryableByName)]
struct Row {
    #[diesel(sql_type = Nullable<Text>)]
    name: Option<String>,
    #[diesel(sql_type = BigInt)]
    total_seconds: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageStat {
    pub name: String,
    pub total_seconds: i64,
}

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub total_time: i64,
    pub top_projects: Vec<UsageStat>,
    pub top_languages: Vec<UsageStat>,
    pub top_oses: Vec<UsageStat>,
    pub top_editors: Vec<UsageStat>,
}

#[derive(QueryableByName, Debug, Clone, Serialize)]
pub struct LanguageCount {
    #[diesel(sql_type = Nullable<Text>)]
    pub language: Option<String>,
    #[diesel(sql_type = BigInt)]
    pub count: i64,
}

#[derive(QueryableByName, Debug, Clone, Serialize)]
pub struct ProjectCount {
    #[diesel(sql_type = Nullable<Text>)]
    pub project: Option<String>,
    #[diesel(sql_type = BigInt)]
    pub count: i64,
}

#[derive(QueryableByName, Debug, Clone, Serialize)]
pub struct DailyActivity {
    #[diesel(sql_type = Date)]
    pub date: chrono::NaiveDate,
    #[diesel(sql_type = BigInt)]
    pub count: i64,
}

#[derive(QueryableByName, Debug, Clone, Serialize)]
pub struct DurationResult {
    #[diesel(sql_type = BigInt)]
    pub total_seconds: i64,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum HeartbeatInput {
    Single(Box<HeartbeatRequest>),
    Multiple(Vec<HeartbeatRequest>),
    Wrapped(WrappedHeartbeatRequest),
}

pub struct DurationInput {
    pub user_id: Option<i32>,
    pub start_date: Option<i64>,
    pub end_date: Option<i64>,
    pub project: Option<String>,
    pub language: Option<String>,
    pub entity: Option<String>,
    pub type_filter: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct WrappedHeartbeatRequest {
    pub heartbeats: Vec<HeartbeatRequest>,
}

impl HeartbeatInput {
    pub fn into_vec(self) -> Vec<HeartbeatRequest> {
        match self {
            HeartbeatInput::Single(heartbeat) => vec![*heartbeat],
            HeartbeatInput::Multiple(heartbeats) => heartbeats,
            HeartbeatInput::Wrapped(wrapped) => wrapped.heartbeats,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct HeartbeatRequest {
    pub entity: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub time: f64,
    pub category: Option<String>,
    pub project: Option<String>,
    pub project_root_count: Option<i32>,
    pub branch: Option<String>,
    pub language: Option<String>,
    pub dependencies: Option<Vec<String>>,
    pub lines: Option<i32>,
    pub line_additions: Option<i32>,
    pub line_deletions: Option<i32>,
    pub lineno: Option<i32>,
    pub cursorpos: Option<i32>,
    pub is_write: Option<bool>,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum HeartbeatApiResponseVariant {
    Single(HeartbeatApiResponse),
    Multiple(Vec<HeartbeatResponse>),
}

#[derive(Serialize, Debug)]
pub struct HeartbeatResponse {
    pub id: String,
    pub entity: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub time: f64,
}

#[derive(Serialize, Debug)]
pub struct HeartbeatApiResponse {
    pub data: HeartbeatResponse,
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = heartbeats)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Heartbeat {
    pub id: i32,
    pub time: i64,
    pub created_at: DateTime<Utc>,
    pub user_id: i32,
    pub entity: String,
    pub type_: String,
    pub ip_address: IpNetwork,
    pub project: Option<String>,
    pub branch: Option<String>,
    pub language: Option<String>,
    pub category: Option<String>,
    pub is_write: Option<bool>,
    pub editor: Option<String>,
    pub operating_system: Option<String>,
    pub machine: Option<String>,
    pub user_agent: String,
    pub lines: Option<i32>,
    pub project_root_count: Option<i32>,
    pub dependencies: Option<Vec<Option<String>>>,
    pub line_additions: Option<i32>,
    pub line_deletions: Option<i32>,
    pub lineno: Option<i32>,
    pub cursorpos: Option<i32>,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = heartbeats)]
pub struct NewHeartbeat {
    pub user_id: i32,
    pub time: i64,
    pub entity: String,
    pub type_: String,
    pub ip_address: IpNetwork,
    pub project: Option<String>,
    pub branch: Option<String>,
    pub language: Option<String>,
    pub category: Option<String>,
    pub is_write: Option<bool>,
    pub editor: Option<String>,
    pub operating_system: Option<String>,
    pub machine: String,
    pub user_agent: String,
    pub lines: Option<i32>,
    pub project_root_count: Option<i32>,
    pub dependencies: Option<Vec<Option<String>>>,
    pub line_additions: Option<i32>,
    pub line_deletions: Option<i32>,
    pub lineno: Option<i32>,
    pub cursorpos: Option<i32>,
}

impl NewHeartbeat {
    pub fn new(
        time: i64, // Unix timestamp
        user_id: i32,
        entity: String,
        type_: String,
        ip_address: IpNetwork,
    ) -> Self {
        Self {
            time,
            user_id,
            entity: truncate_string(entity, MAX_ENTITY_LENGTH),
            type_: truncate_string(type_, MAX_TYPE_LENGTH),
            ip_address,
            project: None,
            branch: None,
            language: None,
            category: None,
            is_write: None,
            editor: None,
            operating_system: None,
            machine: String::new(),
            user_agent: String::new(),
            lines: None,
            project_root_count: None,
            dependencies: None,
            line_additions: None,
            line_deletions: None,
            lineno: None,
            cursorpos: None,
        }
    }

    pub fn from_request(
        request: HeartbeatRequest,
        user_id: i32,
        ip_address: IpNetwork,
        headers: &HeaderMap,
    ) -> Self {
        let time = request.time as i64;

        // extract user agent from headers
        let user_agent = headers
            .get("user-agent")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("")
            .to_string();

        // extract machine name from headers
        let machine = headers
            .get("x-machine-name")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("")
            .to_string();

        // parse user agent to get OS and editor info
        let (operating_system, editor) = match parse_user_agent(user_agent.clone()) {
            Ok((os, ed)) => (Some(os), Some(ed)),
            Err(_) => (None, None),
        };

        // convert dependencies Vec<String> to Vec<Option<String>> if present
        // and limit them to max 50 items and max 254 chars each
        let dependencies = request.dependencies.map(|deps| {
            deps.into_iter()
                .take(50)
                .map(|dep| {
                    let truncated = if dep.len() > 254 {
                        dep.chars().take(254).collect::<String>()
                    } else {
                        dep
                    };
                    Some(truncated)
                })
                .collect()
        });

        // guess language from entity
        let language = {
            let ext = std::path::Path::new(&request.entity)
                .extension()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            if !ext.is_empty() {
                Language::from_extension(ext).map(|lang| lang.name().to_string())
            } else {
                Some("Unknown".to_string())
            }
        };

        // handle test heartbeats
        let type_ = if request.entity == "test.txt" {
            "test".to_string()
        } else {
            request.type_
        };

        Self {
            time,
            user_id,
            entity: truncate_string(request.entity, MAX_ENTITY_LENGTH),
            type_: truncate_string(type_, MAX_TYPE_LENGTH),
            ip_address,
            project: truncate_optional_string(request.project, MAX_PROJECT_LENGTH),
            branch: truncate_optional_string(request.branch, MAX_BRANCH_LENGTH),
            language: truncate_optional_string(language, MAX_LANGUAGE_LENGTH),
            category: truncate_optional_string(request.category, MAX_CATEGORY_LENGTH),
            is_write: request.is_write,
            editor: truncate_optional_string(editor, MAX_EDITOR_LENGTH),
            operating_system: truncate_optional_string(operating_system, MAX_OS_LENGTH),
            machine: truncate_string(machine, MAX_MACHINE_LENGTH),
            user_agent: truncate_string(user_agent, MAX_USER_AGENT_LENGTH),
            lines: request.lines,
            project_root_count: request.project_root_count,
            dependencies,
            line_additions: request.line_additions,
            line_deletions: request.line_deletions,
            lineno: request.lineno,
            cursorpos: request.cursorpos,
        }
    }
}

impl From<Heartbeat> for HeartbeatResponse {
    fn from(heartbeat: Heartbeat) -> Self {
        let id = heartbeat.id.to_string();
        let time = heartbeat.time as f64;

        Self {
            id,
            entity: heartbeat.entity,
            type_: heartbeat.type_,
            time,
        }
    }
}

impl Heartbeat {
    pub fn count_total_heartbeats(conn: &mut PgConnection) -> QueryResult<i64> {
        heartbeats::table.count().get_result(conn)
    }

    pub fn count_heartbeats_last_24h(conn: &mut PgConnection) -> QueryResult<i64> {
        let twenty_four_hours_ago = chrono::Utc::now().timestamp() - 24 * 60 * 60;

        heartbeats::table
            .filter(heartbeats::time.gt(twenty_four_hours_ago))
            .count()
            .get_result(conn)
    }

    pub fn count_heartbeats_last_hour(conn: &mut PgConnection) -> QueryResult<i64> {
        let one_hour_ago = chrono::Utc::now().timestamp() - 60 * 60;

        heartbeats::table
            .filter(heartbeats::time.gt(one_hour_ago))
            .count()
            .get_result(conn)
    }

    pub fn get_top_languages(
        conn: &mut PgConnection,
        limit: i64,
    ) -> QueryResult<Vec<LanguageCount>> {
        diesel::sql_query(
            "SELECT language, COUNT(*) as count 
             FROM heartbeats 
             WHERE language IS NOT NULL 
             GROUP BY language 
             ORDER BY count DESC 
             LIMIT $1",
        )
        .bind::<BigInt, _>(limit)
        .load::<LanguageCount>(conn)
    }

    pub fn get_top_projects(conn: &mut PgConnection, limit: i64) -> QueryResult<Vec<ProjectCount>> {
        diesel::sql_query(
            "SELECT project, COUNT(*) as count 
             FROM heartbeats 
             WHERE project IS NOT NULL 
             GROUP BY project 
             ORDER BY count DESC 
             LIMIT $1",
        )
        .bind::<BigInt, _>(limit)
        .load::<ProjectCount>(conn)
    }

    pub fn get_daily_activity_last_week(
        conn: &mut PgConnection,
    ) -> QueryResult<Vec<DailyActivity>> {
        diesel::sql_query(
            "SELECT DATE(to_timestamp(time)) as date, COUNT(*) as count 
             FROM heartbeats 
             WHERE time >= EXTRACT(EPOCH FROM (NOW() - INTERVAL '7 days'))
             GROUP BY DATE(to_timestamp(time))
             ORDER BY date",
        )
        .load::<DailyActivity>(conn)
    }

    /// Calculate total duration in seconds using SQL with filters
    pub fn get_user_duration_seconds(
        conn: &mut PgConnection,
        duration_input: DurationInput,
    ) -> QueryResult<i64> {
        let base_query = r#"
            WITH capped_diffs AS (
                SELECT CASE
                    WHEN LAG(time) OVER (ORDER BY time) IS NULL THEN 0
                    ELSE LEAST(EXTRACT(EPOCH FROM (to_timestamp(time) - to_timestamp(LAG(time) OVER (ORDER BY time)))), $8)
                END as diff
                FROM heartbeats
                WHERE ($1::int IS NULL OR user_id = $1)
                  AND ($2::bigint IS NULL OR time >= $2)
                  AND ($3::bigint IS NULL OR time <= $3)
                  AND ($4::text IS NULL OR project = $4)
                  AND ($5::text IS NULL OR language = $5)
                  AND ($6::text IS NULL OR entity = $6)
                  AND ($7::text IS NULL OR type = $7)
                  AND time IS NOT NULL
                ORDER BY time ASC
            )
            SELECT COALESCE(SUM(diff), 0)::bigint AS total_seconds
            FROM capped_diffs
            "#;

        let result = diesel::sql_query(base_query)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Int4>, _>(duration_input.user_id)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::BigInt>, _>(
                duration_input.start_date,
            )
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::BigInt>, _>(
                duration_input.end_date,
            )
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(
                duration_input.project.as_deref(),
            )
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(
                duration_input.language.as_deref(),
            )
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(
                duration_input.entity.as_deref(),
            )
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(
                duration_input.type_filter.as_deref(),
            )
            .bind::<diesel::sql_types::Int4, _>(TIMEOUT_SECONDS)
            .get_result::<DurationResult>(conn)?;

        Ok(result.total_seconds)
    }

    /// Calculate total duration in seconds using SQL
    pub fn get_user_total_duration_seconds(
        conn: &mut PgConnection,
        user_id: i32,
    ) -> QueryResult<i64> {
        let result = Self::get_user_duration_seconds(
            conn,
            DurationInput {
                user_id: Some(user_id),
                start_date: None,
                end_date: None,
                project: None,
                language: None,
                entity: None,
                type_filter: None,
            },
        );

        match result {
            Ok(total_seconds) => Ok(total_seconds),
            Err(err) => {
                eprintln!("❌ Error calculating total duration: {}", err);
                Err(err)
            }
        }
    }

    /// Get the count of heartbeats for a user
    pub fn get_user_heartbeat_count(conn: &mut PgConnection, user_id: i32) -> QueryResult<i64> {
        heartbeats::table
            .filter(heartbeats::user_id.eq(user_id))
            .count()
            .get_result(conn)
    }

    /// Get top 10 projects, editors, OSes, and languages by total seconds
    pub fn get_dashboard_stats(
        conn: &mut PgConnection,
        user_id: i32,
    ) -> QueryResult<DashboardStats> {
        // helper closure to run the SQL for a given field
        let mut get_stats = |field: &str| -> QueryResult<Vec<UsageStat>> {
            let sql = format!(
                r#"
                WITH capped_diffs AS (
                    SELECT
                        {field} as name,
                        CASE
                            WHEN LAG(time) OVER (PARTITION BY {field} ORDER BY time) IS NULL THEN 0
                            ELSE LEAST(EXTRACT(EPOCH FROM (to_timestamp(time) - to_timestamp(LAG(time) OVER (PARTITION BY {field} ORDER BY time)))), $2)
                        END as diff
                    FROM heartbeats
                    WHERE {field} IS NOT NULL
                    AND user_id = $1
                    AND time IS NOT NULL
                    ORDER BY time ASC
                )
                SELECT
                    name,
                    COALESCE(SUM(diff), 0)::bigint as total_seconds
                FROM capped_diffs
                GROUP BY name
                ORDER BY total_seconds DESC
                LIMIT 10
                "#,
                field = field,
            );

            let rows: Vec<Row> = diesel::sql_query(sql)
                .bind::<diesel::sql_types::Int4, _>(user_id)
                .bind::<diesel::sql_types::Int4, _>(TIMEOUT_SECONDS)
                .load(conn)?;

            Ok(rows
                .into_iter()
                .map(|row| UsageStat {
                    name: row.name.unwrap_or_else(|| "Unknown".to_string()),
                    total_seconds: row.total_seconds,
                })
                .collect())
        };

        let top_projects = get_stats("project")?;
        let top_editors = get_stats("editor")?;
        let top_oses = get_stats("operating_system")?;
        let top_languages = get_stats("language")?;

        let total_time = Self::get_user_total_duration_seconds(conn, user_id)?;

        Ok(DashboardStats {
            total_time,
            top_projects,
            top_languages,
            top_oses,
            top_editors,
        })
    }
}
