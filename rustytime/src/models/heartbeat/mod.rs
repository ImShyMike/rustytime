use axum::http::HeaderMap;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use diesel::dsl::sql;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Date, Int4, Nullable as SqlNullable, Text, Timestamptz};
use ipnetwork::IpNetwork;
use schemars::JsonSchema;
use serde::de::{self, Deserializer, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::schema::heartbeats::{self};
use crate::utils::http::parse_user_agent;
use crate::utils::time::{
    TimeFormat, get_day_start_utc, get_month_start_date, get_week_start_date,
    human_readable_duration, parse_timezone,
};

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
    /// Calculate all user durations in a time range
    fn calculate_all_user_durations(
        start_date: Timestamptz,
        end_date: Timestamptz,
        timeout_seconds: Int4
    ) -> diesel::sql_types::Array<diesel::sql_types::Record<(Int4, BigInt)>>;
}

diesel::define_sql_function! {
    /// Calculate dashboard statistics (projects, editors, os, languages) in one call
    fn calculate_dashboard_stats(
        user_id: Int4,
        timeout_seconds: Int4,
        limit_count: Int4
    ) -> diesel::sql_types::Array<
        diesel::sql_types::Record<(
            Text,
            SqlNullable<Text>,
            BigInt,
            BigInt,
        )>
    >;
}

diesel::define_sql_function! {
    /// Calculate dashboard statistics with time range filter
    fn calculate_dashboard_stats_by_range(
        user_id: Int4,
        start_time: Timestamptz,
        timeout_seconds: Int4,
        limit_count: Int4
    ) -> diesel::sql_types::Array<
        diesel::sql_types::Record<(
            Text,
            SqlNullable<Text>,
            BigInt,
            BigInt,
        )>
    >;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "lowercase")]
pub enum TimeRange {
    Day,
    Week,
    #[default]
    Month,
    All,
}

impl TimeRange {
    pub fn as_str(&self) -> &'static str {
        match self {
            TimeRange::Day => "day",
            TimeRange::Week => "week",
            TimeRange::Month => "month",
            TimeRange::All => "all",
        }
    }
}

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

/// Convert DateTime<Utc> to f64 timestamp, rounded to 3 decimal places
#[inline(always)]
pub fn datetime_to_f64(time: DateTime<Utc>) -> f64 {
    let raw = time.timestamp() as f64 + time.timestamp_subsec_nanos() as f64 / 1e9;
    round_timestamp(raw)
}

/// Round a timestamp to 3 decimal places (millisecond precision)
#[inline(always)]
fn round_timestamp(timestamp: f64) -> f64 {
    (timestamp * 1000.0).round() / 1000.0
}

/// Convert f64 timestamp to DateTime<Utc>, rounded to millisecond precision
#[inline(always)]
pub fn f64_to_datetime(timestamp: f64) -> DateTime<Utc> {
    let rounded = round_timestamp(timestamp);
    let secs = rounded.trunc() as i64;
    let millis = ((rounded.fract()) * 1000.0).round() as u32;
    DateTime::from_timestamp(secs, millis * 1_000_000).unwrap_or_else(Utc::now)
}

#[repr(i16)]
#[allow(dead_code)]
pub enum SourceType {
    DirectEntry = 0,
    Seeding = 1,
    TestEntry = 2,
    HackatimeImport = 3,
    WakaTimeImport = 4,
}

#[derive(QueryableByName)]
pub struct UserDurationRow {
    #[diesel(sql_type = Int4)]
    pub user_id: i32,
    #[diesel(sql_type = BigInt)]
    pub total_seconds: i64,
}

#[derive(QueryableByName)]
struct CountRow {
    #[diesel(sql_type = BigInt)]
    count: i64,
}

#[derive(QueryableByName)]
struct NullableNameDurationRow {
    #[diesel(sql_type = SqlNullable<Text>)]
    name: Option<String>,
    #[diesel(sql_type = BigInt)]
    total_seconds: i64,
}

#[derive(QueryableByName)]
struct DashboardMetricRow {
    #[diesel(sql_type = Text)]
    metric_type: String,
    #[diesel(sql_type = SqlNullable<Text>)]
    name: Option<String>,
    #[diesel(sql_type = BigInt)]
    total_seconds: i64,
    #[diesel(sql_type = BigInt)]
    total_time: i64,
}

#[derive(Debug, Clone, Serialize, JsonSchema)]
pub struct UsageStat {
    pub name: String,
    pub total_seconds: i64,
    pub text: String,
    pub percent: f32,
}

#[derive(Debug, Clone, Serialize, Default)]
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

#[derive(Deserialize, JsonSchema)]
#[allow(dead_code)]
pub struct HackatimeHeartbeat {
    pub id: i64,
    pub user_id: i32,
    pub branch: Option<String>,
    pub category: Option<String>,
    pub dependencies: Option<Vec<String>>,
    pub editor: Option<String>,
    pub entity: String,
    pub language: Option<String>,
    pub machine: Option<String>,
    pub operating_system: Option<String>,
    pub project: Option<String>,
    #[serde(rename = "type")]
    pub type_: String,
    pub user_agent: Option<String>,
    pub line_additions: Option<i32>,
    pub line_deletions: Option<i32>,
    pub lineno: Option<i32>,
    pub lines: Option<i32>,
    pub cursorpos: Option<i32>,
    pub project_root_count: Option<i32>,
    #[serde(deserialize_with = "deserialize_hackatime_time")]
    pub time: f64,
    pub is_write: Option<bool>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub fields_hash: Option<String>,
    pub source_type: Option<String>,
    pub ip_address: Option<String>,
    pub ysws_program: Option<String>,
    pub deleted_at: Option<String>,
    #[serde(default)]
    pub raw_data: serde_json::Value,
    pub raw_heartbeat_upload_id: Option<i64>,
}

impl HackatimeHeartbeat {
    pub fn to_new_heartbeat(&self, user_id: i32) -> NewHeartbeat {
        let source_type_ = match self.source_type.as_deref() {
            Some("direct_entry") => SourceType::HackatimeImport,
            Some("wakapi_import") => SourceType::WakaTimeImport,
            Some("test_entry") => SourceType::TestEntry,
            _ => SourceType::HackatimeImport,
        };

        let dependencies = self.dependencies.clone().map(|deps| {
            deps.into_iter()
                .take(MAX_DEPENDENCIES)
                .map(|dep| {
                    Some(if dep.len() > MAX_DEPENDENCY_LENGTH {
                        dep.chars().take(MAX_DEPENDENCY_LENGTH).collect::<String>()
                    } else {
                        dep
                    })
                })
                .collect()
        });

        NewHeartbeat {
            user_id,
            project_id: None,
            branch: truncate_optional_string(self.branch.clone(), MAX_BRANCH_LENGTH),
            category: truncate_optional_string(self.category.clone(), MAX_CATEGORY_LENGTH),
            dependencies,
            editor: truncate_optional_string(
                self.editor
                    .clone()
                    .map(|editor| editor.to_ascii_lowercase()),
                MAX_EDITOR_LENGTH,
            ),
            entity: truncate_string(self.entity.clone(), MAX_ENTITY_LENGTH),
            language: truncate_optional_string(self.language.clone(), MAX_LANGUAGE_LENGTH),
            machine: truncate_optional_string(self.machine.clone(), MAX_MACHINE_LENGTH),
            operating_system: truncate_optional_string(
                self.operating_system
                    .clone()
                    .map(|os| os.to_ascii_lowercase()),
                MAX_OS_LENGTH,
            ),
            project: truncate_optional_string(self.project.clone(), MAX_PROJECT_LENGTH),
            type_: truncate_string(self.type_.clone(), MAX_TYPE_LENGTH),
            user_agent: truncate_string(
                self.user_agent.clone().unwrap_or_default(),
                MAX_USER_AGENT_LENGTH,
            ),
            line_additions: self.line_additions,
            line_deletions: self.line_deletions,
            lineno: self.lineno,
            lines: self.lines,
            cursorpos: self.cursorpos,
            project_root_count: self.project_root_count,
            is_write: self.is_write,
            time: f64_to_datetime(self.time),
            ip_address: self
                .ip_address
                .as_deref()
                .unwrap_or("127.0.0.1/32")
                .parse()
                .unwrap_or_else(|_| "127.0.0.1/32".parse().unwrap()),
            source_type: Some(source_type_ as i16),
        }
    }
}

fn deserialize_hackatime_time<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    struct TimeVisitor;

    impl<'de> Visitor<'de> for TimeVisitor {
        type Value = f64;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a numeric timestamp or RFC3339 string")
        }

        fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value)
        }

        fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value as f64)
        }

        fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(value as f64)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            if let Ok(parsed) = value.parse::<f64>() {
                return Ok(parsed);
            }

            if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
                let utc = dt.with_timezone(&Utc);
                return Ok(datetime_to_f64(utc));
            }

            Err(E::custom(format!("invalid Hackatime timestamp: {value}")))
        }
    }

    deserializer.deserialize_any(TimeVisitor)
}

#[derive(Deserialize, Debug, JsonSchema)]
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

#[derive(Deserialize, Debug, JsonSchema)]
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

#[derive(Deserialize, Debug, JsonSchema)]
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
    pub plugin: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Serialize, Debug, JsonSchema)]
#[serde(untagged)]
pub enum HeartbeatApiResponseVariant {
    Single(HeartbeatApiResponse),
    Multiple(HeartbeatBulkApiResponse),
}

#[derive(Serialize, Debug, JsonSchema)]
pub struct HeartbeatResponse {
    pub id: i64,
    pub entity: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub time: f64,
}

#[derive(Serialize, Debug, JsonSchema)]
pub struct HeartbeatApiResponse {
    pub data: HeartbeatResponse,
}

#[derive(Serialize, Debug, JsonSchema)]
pub struct HeartbeatBulkApiResponse {
    pub responses: Vec<BulkResponseItem>,
}

#[derive(Serialize, Debug, JsonSchema)]
pub struct BulkResponseItem(pub HeartbeatResponse, pub u16);

#[derive(Queryable, QueryableByName, Selectable, Serialize, Deserialize, Debug, Clone)]
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
    pub source_type: Option<i16>,
    pub plugin: Option<String>,
    pub user_agent: Option<String>,
}

impl SanitizedHeartbeatRequest {
    pub fn from_request(request: HeartbeatRequest) -> Self {
        // Convert timestamp (seconds since epoch) to DateTime<Utc>, rounded to millisecond precision
        let time = f64_to_datetime(request.time);

        // Handle test heartbeats
        let source_type_ = if request.entity == "test.txt" {
            SourceType::TestEntry
        } else {
            SourceType::DirectEntry
        };

        // Parse the category
        let category = if let Some(cat) = request.category {
            Some(cat)
        } else if request.type_ == "domain" || request.type_ == "url" {
            Some("browsing".to_string())
        } else {
            Some("coding".to_string())
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
            type_: truncate_string(request.type_, MAX_TYPE_LENGTH),
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
            source_type: Some(source_type_ as i16),
            user_agent: request.user_agent,
            plugin: request.plugin,
        }
    }

    pub fn into_new_heartbeat(
        mut self,
        user_id: i32,
        ip_address: IpNetwork,
        headers: &HeaderMap,
    ) -> NewHeartbeat {
        // Get user agent with fallbacks (user_agent -> header -> plugin)
        let user_agent = self
            .user_agent
            .take()
            .or_else(|| {
                headers
                    .get("user-agent")
                    .and_then(|value| value.to_str().ok())
                    .map(|value| value.to_string())
            })
            .or_else(|| self.plugin.take())
            .unwrap_or_default();

        // Extract machine name from headers
        let machine = headers
            .get("x-machine-name")
            .and_then(|value| value.to_str().ok())
            .map(|s| s.to_string());

        let (operating_system, editor) = if user_agent.is_empty() {
            (None, None)
        } else {
            // Parse user agent to get OS and editor info
            match parse_user_agent(user_agent.clone()) {
                Ok((os, ed)) => (os, ed),
                Err(_) => (None, None),
            }
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
            source_type: self.source_type,
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
    pub fn total_heartbeat_count_estimate(conn: &mut PgConnection) -> QueryResult<i64> {
        diesel::sql_query("SELECT * FROM approximate_row_count('heartbeats') AS count")
            .get_result::<CountRow>(conn)
            .map(|res| res.count)
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
        let now = chrono::Utc::now();

        let start_today = now.date_naive().and_hms_opt(0, 0, 0).unwrap();
        let seven_days_ago = start_today - chrono::Duration::days(7);

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
        diesel::sql_query(
            "SELECT user_id, total_seconds \
             FROM calculate_all_user_durations($1, $2, $3)",
        )
        .bind::<Timestamptz, _>(start_time)
        .bind::<Timestamptz, _>(end_time)
        .bind::<Int4, _>(TIMEOUT_SECONDS)
        .load::<UserDurationRow>(conn)
    }

    /// Get the count of heartbeats for a user
    pub fn get_user_heartbeat_count(conn: &mut PgConnection, user_id: i32) -> QueryResult<i64> {
        heartbeats::table
            .filter(heartbeats::user_id.eq(user_id))
            .count()
            .get_result(conn)
    }

    /// Get the count of heartbeats for a user in a specific time range
    pub fn get_user_heartbeat_count_by_range(
        conn: &mut PgConnection,
        user_id: i32,
        range: TimeRange,
        user_timezone: &str,
    ) -> QueryResult<i64> {
        let tz = parse_timezone(user_timezone);
        let now = Utc::now();

        match Self::start_boundary_utc(range, tz, now) {
            Some(start) => heartbeats::table
                .filter(heartbeats::user_id.eq(user_id))
                .filter(heartbeats::time.ge(start))
                .count()
                .get_result(conn),
            None => Self::get_user_heartbeat_count(conn, user_id),
        }
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

    /// Compute the start boundary in UTC for a given time range based on user's timezone
    fn start_boundary_utc(
        range: TimeRange,
        tz: Tz,
        now_utc: DateTime<Utc>,
    ) -> Option<DateTime<Utc>> {
        if range == TimeRange::All {
            return None;
        }

        let today = now_utc.with_timezone(&tz).date_naive();

        let start_date = match range {
            TimeRange::Day => today,
            TimeRange::Week => get_week_start_date(today),
            TimeRange::Month => get_month_start_date(today),
            TimeRange::All => unreachable!(),
        };

        Some(get_day_start_utc(start_date, tz))
    }

    /// Get dashboard stats filtered by time range (day, week, month, all)
    pub fn get_dashboard_stats_by_range(
        conn: &mut PgConnection,
        user_id: i32,
        range: TimeRange,
        user_timezone: &str,
    ) -> QueryResult<DashboardStats> {
        let tz = parse_timezone(user_timezone);
        let now = Utc::now();

        let filtered_rows: Vec<DashboardMetricRow> = match Self::start_boundary_utc(range, tz, now)
        {
            Some(start_time) => diesel::sql_query(
                "SELECT metric_type, name, total_seconds, total_time \
                     FROM calculate_dashboard_stats_by_range($1, $2, $3, $4)",
            )
            .bind::<Int4, _>(user_id)
            .bind::<Timestamptz, _>(start_time)
            .bind::<Int4, _>(TIMEOUT_SECONDS)
            .bind::<Int4, _>(10)
            .load(conn)?,
            None => diesel::sql_query(
                "SELECT metric_type, name, total_seconds, total_time \
                     FROM calculate_dashboard_stats($1, $2, $3) \
                     WHERE total_seconds > 0 OR metric_type = 'total_time'",
            )
            .bind::<Int4, _>(user_id)
            .bind::<Int4, _>(TIMEOUT_SECONDS)
            .bind::<Int4, _>(10)
            .load(conn)?,
        };

        let mut total_time: i64 = 0;
        let mut project_rows = Vec::new();
        let mut editor_rows = Vec::new();
        let mut os_rows = Vec::new();
        let mut language_rows = Vec::new();

        for row in filtered_rows {
            match row.metric_type.as_str() {
                "total_time" => total_time = row.total_time,
                "project" => project_rows.push(NullableNameDurationRow {
                    name: row.name,
                    total_seconds: row.total_seconds,
                }),
                "editor" => editor_rows.push(NullableNameDurationRow {
                    name: row.name,
                    total_seconds: row.total_seconds,
                }),
                "operating_system" => os_rows.push(NullableNameDurationRow {
                    name: row.name,
                    total_seconds: row.total_seconds,
                }),
                "language" => language_rows.push(NullableNameDurationRow {
                    name: row.name,
                    total_seconds: row.total_seconds,
                }),
                _ => {}
            }
        }

        Ok(DashboardStats {
            total_time,
            top_projects: Self::map_usage_stats(project_rows, total_time),
            top_languages: Self::map_usage_stats(language_rows, total_time),
            top_oses: Self::map_usage_stats(os_rows, total_time),
            top_editors: Self::map_usage_stats(editor_rows, total_time),
        })
    }
}

#[cfg(test)]
mod tests;
