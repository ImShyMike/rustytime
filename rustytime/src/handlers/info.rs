use axum::Json;
use schemars::JsonSchema;
use serde::Serialize;

use crate::{START_TIME, utils::env::is_production_env};

#[derive(Serialize, JsonSchema)]
pub struct InfoResponse {
    name: String,
    version: String,
    git_sha: String,
    environment: String,
    uptime: String,
    build_profile: String,
    build_target: String,
    build_timestamp: String,
    repository_url: String,
}

/// Info endpoint
pub async fn info() -> Json<InfoResponse> {
    Json(InfoResponse {
        name: env!("CARGO_PKG_NAME").to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        git_sha: env!("GIT_SHA").to_string(),
        environment: if is_production_env() {
            "production".to_string()
        } else {
            "development".to_string()
        },
        uptime: START_TIME.elapsed().as_secs().to_string(),
        build_profile: if cfg!(debug_assertions) {
            "debug".to_string()
        } else {
            "release".to_string()
        },
        build_target: env!("TARGET").to_string(),
        build_timestamp: env!("BUILD_TIMESTAMP").to_string(),
        repository_url: "https://github.com/ImShyMike/rustytime".to_string(),
    })
}
