use chrono::{DateTime, Utc};
use diesel::prelude::*;
use ipnetwork::IpNetwork;
use serde::{Deserialize, Serialize};

use crate::schema::heartbeats;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum HeartbeatInput {
    Single(Box<HeartbeatRequest>),
    Multiple(Vec<HeartbeatRequest>),
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
            machine: None,
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

    pub fn from_request(request: HeartbeatRequest, user_id: i32, ip_address: IpNetwork) -> Self {
        let created_at = DateTime::from_timestamp(
            request.time as i64,
            (request.time.fract() * 1_000_000_000.0) as u32,
        )
        .unwrap_or_else(Utc::now);

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

        Self {
            created_at,
            user_id,
            entity: request.entity,
            type_: request.type_,
            ip_address,
            project: request.project,
            branch: request.branch,
            language: request.language,
            category: request.category,
            is_write: request.is_write,
            editor: None,
            operating_system: None,
            machine: None,
            user_agent: String::new(),
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
