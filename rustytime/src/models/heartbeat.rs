use axum::http::HeaderMap;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Date, Nullable, Text};
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};

use crate::schema::heartbeats;
use crate::utils::http::parse_user_agent;

const TIMEOUT_SECONDS: i32 = 120; // 2 minutes in seconds

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
}

pub struct DurationInput {
    pub user_id: Option<i32>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub project: Option<String>,
    pub language: Option<String>,
    pub entity: Option<String>,
    pub type_filter: Option<String>,
}

impl HeartbeatInput {
    pub fn into_vec(self) -> Vec<HeartbeatRequest> {
        match self {
            HeartbeatInput::Single(heartbeat) => vec![*heartbeat],
            HeartbeatInput::Multiple(heartbeats) => heartbeats,
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct HeartbeatRequest {
    pub entity: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub category: Option<String>,
    pub time: f64,
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
    Multiple(HeartbeatsApiResponse),
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

#[derive(Serialize, Debug)]
pub struct HeartbeatsApiResponse {
    pub data: Vec<HeartbeatResponse>,
}

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = heartbeats)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Heartbeat {
    pub id: i32,
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
        created_at: DateTime<Utc>,
        user_id: i32,
        entity: String,
        type_: String,
        ip_address: IpNetwork,
    ) -> Self {
        Self {
            created_at,
            user_id,
            entity,
            type_,
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
        let created_at = DateTime::from_timestamp(
            request.time as i64,
            (request.time.fract() * 1_000_000_000.0) as u32,
        )
        .unwrap_or_else(Utc::now);

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

        // handle test heartbeats
        let type_ = if request.entity == "test.txt" {
            "test".to_string()
        } else {
            request.type_
        };

        Self {
            created_at,
            user_id,
            entity: request.entity,
            type_,
            ip_address,
            project: request.project,
            branch: request.branch,
            language: request.language,
            category: request.category,
            is_write: request.is_write,
            editor,
            operating_system,
            machine,
            user_agent,
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
        let time = heartbeat.created_at.timestamp() as f64
            + heartbeat.created_at.timestamp_subsec_nanos() as f64 / 1_000_000_000.0;

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
        use chrono::Duration;
        let twenty_four_hours_ago = Utc::now() - Duration::hours(24);

        heartbeats::table
            .filter(heartbeats::created_at.gt(twenty_four_hours_ago))
            .count()
            .get_result(conn)
    }

    pub fn count_heartbeats_last_hour(conn: &mut PgConnection) -> QueryResult<i64> {
        use chrono::Duration;
        let one_hour_ago = Utc::now() - Duration::hours(1);

        heartbeats::table
            .filter(heartbeats::created_at.gt(one_hour_ago))
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
            "SELECT DATE(created_at) as date, COUNT(*) as count 
             FROM heartbeats 
             WHERE created_at >= NOW() - INTERVAL '7 days' 
             GROUP BY DATE(created_at) 
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
            WITH ordered_heartbeats AS (
                SELECT created_at
                FROM heartbeats
                WHERE ($1::int IS NULL OR user_id = $1)
                  AND ($2::timestamptz IS NULL OR created_at >= $2)
                  AND ($3::timestamptz IS NULL OR created_at <= $3)
                  AND ($4::text IS NULL OR project = $4)
                  AND ($5::text IS NULL OR language = $5)
                  AND ($6::text IS NULL OR entity = $6)
                  AND ($7::text IS NULL OR type_ = $7)
                ORDER BY created_at ASC
            ),
            time_diffs AS (
                SELECT 
                    EXTRACT(EPOCH FROM (
                        LEAD(created_at) OVER (ORDER BY created_at) - created_at
                    ))::integer AS diff_seconds
                FROM ordered_heartbeats
            )
            SELECT COALESCE(SUM(
                CASE 
                    WHEN diff_seconds IS NULL OR diff_seconds < 0 THEN 0
                    WHEN diff_seconds > $8 THEN $8
                    ELSE diff_seconds
                END
            ), 0) AS total_seconds
            FROM time_diffs
            "#;

        let result = diesel::sql_query(base_query)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Int4>, _>(duration_input.user_id)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(
                duration_input.start_date,
            )
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(
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
        let result = diesel::sql_query(
            r#"
            WITH ordered_heartbeats AS (
                SELECT created_at
                FROM heartbeats
                WHERE user_id = $1
                ORDER BY created_at ASC
            ),
            time_diffs AS (
                SELECT 
                    EXTRACT(EPOCH FROM (
                        LEAD(created_at) OVER (ORDER BY created_at) - created_at
                    ))::integer AS diff_seconds
                FROM ordered_heartbeats
            )
            SELECT COALESCE(SUM(
                CASE 
                    WHEN diff_seconds IS NULL OR diff_seconds < 0 THEN 0
                    WHEN diff_seconds > $2 THEN $2
                    ELSE diff_seconds
                END
            ), 0) AS total_seconds
            FROM time_diffs
            "#,
        )
        .bind::<diesel::sql_types::Int4, _>(user_id)
        .bind::<diesel::sql_types::Int4, _>(TIMEOUT_SECONDS)
        .get_result::<DurationResult>(conn)?;

        Ok(result.total_seconds)
    }

    /// Get the count of heartbeats for a user
    pub fn get_user_heartbeat_count(conn: &mut PgConnection, user_id: i32) -> QueryResult<i64> {
        heartbeats::table
            .filter(heartbeats::user_id.eq(user_id))
            .count()
            .get_result(conn)
    }
}
