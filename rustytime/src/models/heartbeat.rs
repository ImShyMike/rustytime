use axum::http::HeaderMap;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Date, Int4, Nullable as SqlNullable, Text, Timestamptz};
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};

use crate::schema::heartbeats;
use crate::utils::http::parse_user_agent;
use crate::utils::time::{TimeFormat, human_readable_duration};

diesel::define_sql_function! {
    /// Calculate user duration with filters
    #[allow(clippy::too_many_arguments)]
    fn calculate_user_duration(
        user_id: SqlNullable<Int4>,
        start_date: SqlNullable<Timestamptz>,
        end_date: SqlNullable<Timestamptz>,
        project: SqlNullable<Text>,
        language: SqlNullable<Text>,
        entity: SqlNullable<Text>,
        type_filter: SqlNullable<Text>,
        timeout_seconds: Int4
    ) -> BigInt;
}

diesel::define_sql_function! {
    /// Get field statistics (language, editor, OS, etc.)
    fn calculate_field_stats(
        user_id: Int4,
        field_name: Text,
        timeout_seconds: Int4,
        limit_count: Int4
    ) -> diesel::sql_types::Array<diesel::sql_types::Record<(Text, BigInt)>>;
}

diesel::define_sql_function! {
    /// Get project statistics with alias resolution
    fn calculate_project_stats_with_aliases(
        user_id: Int4,
        timeout_seconds: Int4,
        limit_count: Int4
    ) -> diesel::sql_types::Array<diesel::sql_types::Record<(Text, BigInt)>>;
}

diesel::define_sql_function! {
    /// Calculate all user durations in a time range
    fn calculate_all_user_durations(
        start_date: Timestamptz,
        end_date: Timestamptz,
        timeout_seconds: Int4
    ) -> diesel::sql_types::Array<diesel::sql_types::Record<(Int4, BigInt)>>;
}

pub const TIMEOUT_SECONDS: i32 = 120; // 2 minutes in seconds

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
const MAX_DEPENDENCIES: usize = 50;
const MAX_DEPENDENCY_LENGTH: usize = 254;

/// Truncate a string to the specified maximum length, respecting UTF-8 boundaries
#[inline(always)]
fn truncate_string(mut s: String, max_length: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_length {
        s
    } else {
        let byte_idx = s
            .char_indices()
            .nth(max_length)
            .map(|(idx, _)| idx)
            .unwrap_or(s.len());
        s.truncate(byte_idx);
        s.shrink_to_fit();
        s
    }
}

/// Truncate an optional string to the specified maximum length
#[inline(always)]
fn truncate_optional_string(s: Option<String>, max_length: usize) -> Option<String> {
    s.map(|s| truncate_string(s, max_length))
}

/// Convert DateTime<Utc> to f64 timestamp
#[inline(always)]
pub fn datetime_to_f64(time: DateTime<Utc>) -> f64 {
    time.timestamp() as f64 + time.timestamp_subsec_nanos() as f64 / 1e9
}

#[repr(i16)]
#[allow(dead_code)]
pub enum SourceType {
    DirectEntry = 0,
    Import = 1,
    WakaTimeImport = 2,
    Seeding = 3,
}

#[derive(QueryableByName)]
pub struct UserDurationRow {
    #[diesel(sql_type = Int4)]
    pub user_id: i32,
    #[diesel(sql_type = BigInt)]
    pub total_seconds: i64,
}

#[derive(QueryableByName)]
struct NullableNameDurationRow {
    #[diesel(sql_type = SqlNullable<Text>)]
    name: Option<String>,
    #[diesel(sql_type = BigInt)]
    total_seconds: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageStat {
    pub name: String,
    pub total_seconds: i64,
    pub text: String,
    pub percent: f32,
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
pub struct DailyActivity {
    #[diesel(sql_type = Date)]
    pub date: chrono::NaiveDate,
    #[diesel(sql_type = BigInt)]
    pub count: i64,
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
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
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
    Multiple(HeartbeatBulkApiResponse),
}

#[derive(Serialize, Debug)]
pub struct HeartbeatResponse {
    pub id: i64,
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
pub struct HeartbeatBulkApiResponse {
    pub responses: Vec<BulkResponseItem>,
}

#[derive(Serialize, Debug)]
pub struct BulkResponseItem(pub HeartbeatResponse, pub u16);

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = heartbeats)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Heartbeat {
    pub id: i64,
    pub time: DateTime<Utc>,
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
    pub source_type: Option<i16>,
    pub project_id: Option<i32>,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = heartbeats)]
pub struct NewHeartbeat {
    pub user_id: i32,
    pub time: DateTime<Utc>,
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
    pub source_type: Option<i16>,
    pub project_id: Option<i32>,
}

#[derive(Serialize)]
pub struct SanitizedHeartbeatRequest {
    pub entity: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub time: DateTime<Utc>,
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

impl SanitizedHeartbeatRequest {
    pub fn from_request(request: HeartbeatRequest) -> Self {
        // Convert timestamp (seconds since epoch) to DateTime<Utc>
        let time = DateTime::from_timestamp(
            request.time.trunc() as i64,
            (request.time.fract() * 1e9) as u32,
        )
        .unwrap_or_else(Utc::now);

        // Handle test heartbeats
        let type_ = if request.entity == "test.txt" {
            "test".to_string()
        } else {
            request.type_
        };

        // Parse the category
        let category = if let Some(cat) = request.category {
            Some(cat)
        } else if type_ == "domain" || type_ == "url" {
            Some("browsing".to_string())
        } else if type_ == "file" && request.language.is_some() {
            Some("coding".to_string())
        } else {
            None
        };

        // Convert dependencies and apply limits
        let dependencies = request.dependencies.map(|deps| {
            deps.into_iter()
                .take(MAX_DEPENDENCIES)
                .map(|dep| {
                    if dep.len() > MAX_DEPENDENCY_LENGTH {
                        dep.chars().take(MAX_DEPENDENCY_LENGTH).collect::<String>()
                    } else {
                        dep
                    }
                })
                .collect()
        });

        Self {
            entity: truncate_string(request.entity, MAX_ENTITY_LENGTH),
            type_: truncate_string(type_, MAX_TYPE_LENGTH),
            time,
            category: truncate_optional_string(category, MAX_CATEGORY_LENGTH),
            project: truncate_optional_string(request.project, MAX_PROJECT_LENGTH),
            project_root_count: request.project_root_count,
            branch: truncate_optional_string(request.branch, MAX_BRANCH_LENGTH),
            language: truncate_optional_string(request.language, MAX_LANGUAGE_LENGTH),
            dependencies,
            lines: request.lines,
            line_additions: request.line_additions,
            line_deletions: request.line_deletions,
            lineno: request.lineno,
            cursorpos: request.cursorpos,
            is_write: request.is_write,
        }
    }

    pub fn into_new_heartbeat(
        self,
        user_id: i32,
        ip_address: IpNetwork,
        headers: &HeaderMap,
    ) -> NewHeartbeat {
        // Extract user agent from headers
        let user_agent = headers
            .get("user-agent")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("")
            .to_string();

        // Extract machine name from headers
        let machine = headers
            .get("x-machine-name")
            .and_then(|value| value.to_str().ok())
            .map(|s| s.to_string());

        // Parse user agent to get OS and editor info
        let (operating_system, editor) = match parse_user_agent(user_agent.clone()) {
            Ok((os, ed)) => (Some(os), Some(ed)),
            Err(_) => (None, None),
        };

        // Convert dependencies Vec<String> to Vec<Option<String>> if present
        let dependencies = self
            .dependencies
            .map(|deps| deps.into_iter().map(Some).collect());

        NewHeartbeat {
            time: self.time,
            user_id,
            entity: self.entity,
            type_: self.type_,
            ip_address,
            project: self.project,
            branch: self.branch,
            language: self.language,
            category: self.category,
            is_write: self.is_write,
            editor: truncate_optional_string(editor, MAX_EDITOR_LENGTH),
            operating_system: truncate_optional_string(operating_system, MAX_OS_LENGTH),
            machine: truncate_optional_string(machine, MAX_MACHINE_LENGTH),
            user_agent: truncate_string(user_agent, MAX_USER_AGENT_LENGTH),
            lines: self.lines,
            project_root_count: self.project_root_count,
            dependencies,
            line_additions: self.line_additions,
            line_deletions: self.line_deletions,
            lineno: self.lineno,
            cursorpos: self.cursorpos,
            source_type: Some(SourceType::DirectEntry as i16),
            project_id: None,
        }
    }
}

impl NewHeartbeat {
    pub fn new(
        time: DateTime<Utc>,
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
            machine: None,
            user_agent: String::new(),
            lines: None,
            project_root_count: None,
            dependencies: None,
            line_additions: None,
            line_deletions: None,
            lineno: None,
            cursorpos: None,
            source_type: None,
            project_id: None,
        }
    }

    pub fn from_request(
        request: HeartbeatRequest,
        user_id: i32,
        ip_address: IpNetwork,
        headers: &HeaderMap,
    ) -> Self {
        let sanitized = SanitizedHeartbeatRequest::from_request(request);
        sanitized.into_new_heartbeat(user_id, ip_address, headers)
    }
}

impl From<Heartbeat> for HeartbeatResponse {
    fn from(heartbeat: Heartbeat) -> Self {
        Self {
            id: heartbeat.id,
            entity: heartbeat.entity,
            type_: heartbeat.type_,
            time: datetime_to_f64(heartbeat.time),
        }
    }
}

impl From<(i64, NewHeartbeat)> for HeartbeatResponse {
    fn from((id, heartbeat): (i64, NewHeartbeat)) -> Self {
        Self {
            id,
            entity: heartbeat.entity,
            type_: heartbeat.type_,
            time: datetime_to_f64(heartbeat.time),
        }
    }
}

impl From<Heartbeat> for BulkResponseItem {
    fn from(heartbeat: Heartbeat) -> Self {
        let response = HeartbeatResponse::from(heartbeat);
        BulkResponseItem(response, 201)
    }
}

impl Heartbeat {
    pub fn count_total_heartbeats(conn: &mut PgConnection) -> QueryResult<i64> {
        heartbeats::table.count().get_result(conn)
    }

    pub fn count_heartbeats_last_24h(conn: &mut PgConnection) -> QueryResult<i64> {
        let twenty_four_hours_ago = chrono::Utc::now() - chrono::Duration::hours(24);

        heartbeats::table
            .filter(heartbeats::time.gt(twenty_four_hours_ago))
            .count()
            .get_result(conn)
    }

    pub fn count_heartbeats_last_hour(conn: &mut PgConnection) -> QueryResult<i64> {
        let one_hour_ago = chrono::Utc::now() - chrono::Duration::hours(1);

        heartbeats::table
            .filter(heartbeats::time.gt(one_hour_ago))
            .count()
            .get_result(conn)
    }

    pub fn get_daily_activity_last_week(
        conn: &mut PgConnection,
    ) -> QueryResult<Vec<DailyActivity>> {
        use diesel::dsl::*;
        use diesel::sql_types::*;

        let seven_days_ago = chrono::Utc::now() - chrono::Duration::days(7);

        // Use Diesel's select with sql() for date functions not in DSL
        heartbeats::table
            .filter(heartbeats::time.ge(seven_days_ago))
            .select((sql::<Date>("DATE(time)"), sql::<BigInt>("COUNT(*)")))
            .group_by(sql::<Date>("DATE(time)"))
            .order_by(sql::<Date>("DATE(time)"))
            .load::<(chrono::NaiveDate, i64)>(conn)
            .map(|rows| {
                rows.into_iter()
                    .map(|(date, count)| DailyActivity { date, count })
                    .collect()
            })
    }

    /// Calculate total duration in seconds using database function
    pub fn get_user_duration_seconds(
        conn: &mut PgConnection,
        duration_input: DurationInput,
    ) -> QueryResult<i64> {
        // Call database function using Diesel's define_sql_function!
        diesel::select(calculate_user_duration(
            duration_input.user_id,
            duration_input.start_date,
            duration_input.end_date,
            duration_input.project.as_deref(),
            duration_input.language.as_deref(),
            duration_input.entity.as_deref(),
            duration_input.type_filter.as_deref(),
            TIMEOUT_SECONDS,
        ))
        .get_result(conn)
    }

    /// Calculate total durations for all users between start_time and end_time
    pub fn get_all_user_durations(
        conn: &mut PgConnection,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> QueryResult<Vec<UserDurationRow>> {
        // Call the database function using parameter binding to avoid malformed SQL
        diesel::sql_query(
            "SELECT user_id, total_seconds \
             FROM calculate_all_user_durations($1, $2, $3)",
        )
        .bind::<Timestamptz, _>(start_time)
        .bind::<Timestamptz, _>(end_time)
        .bind::<Int4, _>(TIMEOUT_SECONDS)
        .load::<UserDurationRow>(conn)
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

    /// Get top 10 projects by total seconds
    fn get_project_stats_with_aliases(
        conn: &mut PgConnection,
        user_id: i32,
        total_time: i64,
    ) -> QueryResult<Vec<UsageStat>> {
        let rows: Vec<NullableNameDurationRow> = diesel::sql_query(
            "SELECT name, total_seconds \
             FROM calculate_project_stats_with_aliases($1, $2, $3)",
        )
        .bind::<Int4, _>(user_id)
        .bind::<Int4, _>(TIMEOUT_SECONDS)
        .bind::<Int4, _>(10)
        .load(conn)?;

        Ok(Self::map_usage_stats(rows, total_time))
    }

    fn load_field_stats(
        conn: &mut PgConnection,
        user_id: i32,
        field: &str,
    ) -> QueryResult<Vec<NullableNameDurationRow>> {
        diesel::sql_query(
            "SELECT name, total_seconds \
             FROM calculate_field_stats($1, $2, $3, $4)",
        )
        .bind::<Int4, _>(user_id)
        .bind::<Text, _>(field)
        .bind::<Int4, _>(TIMEOUT_SECONDS)
        .bind::<Int4, _>(10)
        .load(conn)
    }

    fn map_usage_stats(rows: Vec<NullableNameDurationRow>, total_time: i64) -> Vec<UsageStat> {
        rows.into_iter()
            .map(|row| UsageStat {
                name: row.name.unwrap_or_else(|| "Unknown".to_string()),
                total_seconds: row.total_seconds,
                text: human_readable_duration(row.total_seconds, TimeFormat::HourMinute)
                    .human_readable,
                percent: if total_time > 0 {
                    ((row.total_seconds as f32 / total_time as f32) * 10000.0).round() / 100.0
                } else {
                    0.0
                },
            })
            .collect()
    }

    /// Get top 10 projects, editors, OSes, and languages by total seconds
    pub fn get_dashboard_stats(
        conn: &mut PgConnection,
        user_id: i32,
    ) -> QueryResult<DashboardStats> {
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            let total_time = Self::get_user_total_duration_seconds(conn, user_id)?;
            let top_projects = Self::get_project_stats_with_aliases(conn, user_id, total_time)?;

            let mut get_stats = |field: &str| -> QueryResult<Vec<UsageStat>> {
                let rows = Self::load_field_stats(conn, user_id, field)?;
                Ok(Self::map_usage_stats(rows, total_time))
            };

            let top_editors = get_stats("editor")?;
            let top_oses = get_stats("operating_system")?;
            let top_languages = get_stats("language")?;

            Ok(DashboardStats {
                total_time,
                top_projects,
                top_languages,
                top_oses,
                top_editors,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_request() -> HeartbeatRequest {
        HeartbeatRequest {
            entity: "example.txt".to_string(),
            type_: "file".to_string(),
            time: 1_700_000_000.123456,
            category: None,
            project: Some("ExampleProject".to_string()),
            project_root_count: Some(1),
            branch: Some("main".to_string()),
            language: Some("Rust".to_string()),
            dependencies: Some(vec!["dep1".to_string(), "dep2".to_string()]),
            lines: Some(100),
            line_additions: Some(10),
            line_deletions: Some(5),
            lineno: Some(42),
            cursorpos: Some(128),
            is_write: Some(true),
        }
    }

    #[test]
    fn truncates_strings_without_breaking_utf8_boundaries() {
        let input = "😀😀😀😀😀😀😀😀😀😀".to_string();
        let truncated = truncate_string(input, 7);
        assert_eq!(truncated.chars().count(), 7);
    }

    #[test]
    fn truncates_optional_strings_when_present() {
        let input = Some("Hello 😀, World!".to_string());
        let truncated = truncate_optional_string(input, 10);
        assert_eq!(truncated.unwrap().chars().count(), 10);
    }

    #[test]
    fn converts_datetime_to_f64_with_nanosecond_precision() {
        let dt =
            DateTime::<Utc>::from_timestamp(1_700_000_000, 987_654_321).expect("valid timestamp");

        let converted = datetime_to_f64(dt);
        let expected = 1_700_000_000f64 + 987_654_321f64 / 1e9;
        let diff = (converted - expected).abs();

        assert!(
            diff <= 1e-12,
            "expected {expected}, got {converted}, diff {diff}"
        );
    }

    #[test]
    fn sanitized_heartbeat_request_infers_defaults() {
        let request = sample_request();
        let sanitized = SanitizedHeartbeatRequest::from_request(request);
        assert_eq!(sanitized.category.unwrap(), "coding");
    }

    #[test]
    fn sanitized_heartbeat_into_new_heartbeat_maps_headers() {
        let request = sample_request();
        let sanitized = SanitizedHeartbeatRequest::from_request(request);
        let headers = HeaderMap::new();
        let new_heartbeat = sanitized.into_new_heartbeat(1, "1.1.1.1".parse().unwrap(), &headers);
        assert_eq!(new_heartbeat.user_id, 1);
        assert_eq!(new_heartbeat.entity, "example.txt".to_string());
        assert_eq!(new_heartbeat.type_, "file".to_string());
        assert_eq!(new_heartbeat.ip_address, "1.1.1.1".parse().unwrap());
    }

    #[test]
    fn new_heartbeat_constructor_applies_truncation() {
        let entity = "a".repeat(MAX_ENTITY_LENGTH + 10);
        let type_ = "b".repeat(MAX_TYPE_LENGTH + 10);
        let new_heartbeat = NewHeartbeat::new(
            Utc::now(),
            1,
            entity.clone(),
            type_.clone(),
            "1.1.1.1".parse().unwrap(),
        );
        assert_eq!(new_heartbeat.entity.chars().count(), MAX_ENTITY_LENGTH);
        assert_eq!(new_heartbeat.type_.chars().count(), MAX_TYPE_LENGTH);
    }

    #[test]
    fn new_heartbeat_from_request_round_trips_request_payload() {
        let request = sample_request();
        let headers = HeaderMap::new();
        let new_heartbeat =
            NewHeartbeat::from_request(request, 1, "1.1.1.1".parse().unwrap(), &headers);
        assert_eq!(new_heartbeat.entity, "example.txt".to_string());
        assert_eq!(new_heartbeat.type_, "file".to_string());
        assert_eq!(new_heartbeat.project.unwrap(), "ExampleProject".to_string());
        assert_eq!(new_heartbeat.language.unwrap(), "Rust".to_string());
        assert_eq!(new_heartbeat.lines.unwrap(), 100);
    }

    #[test]
    fn heartbeat_response_conversion_retains_entity_details() {
        let heartbeat = Heartbeat {
            id: 1,
            time: Utc::now(),
            created_at: Utc::now(),
            user_id: 1,
            entity: "test.txt".to_string(),
            type_: "file".to_string(),
            ip_address: "127.0.0.1/32".parse().unwrap(),
            project: None,
            branch: None,
            category: None,
            cursorpos: None,
            dependencies: None,
            editor: None,
            is_write: None,
            language: None,
            line_additions: None,
            line_deletions: None,
            lines: None,
            machine: None,
            operating_system: None,
            project_id: None,
            project_root_count: None,
            user_agent: "".to_string(),
            lineno: None,
            source_type: None,
        };
        let response = HeartbeatResponse::from(heartbeat.clone());
        assert_eq!(response.id, heartbeat.id);
    }
}
