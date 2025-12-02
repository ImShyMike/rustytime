#![cfg(feature = "seed")]

use crate::db::connection::DbPool;
use crate::handlers::api::user::store_heartbeats_in_db_count_only;
use crate::models::heartbeat::{NewHeartbeat, SourceType};
use crate::models::user::User;
use chrono::Utc;
use ipnetwork::{IpNetwork, Ipv4Network};
use rand::Rng;
use rand::prelude::IndexedRandom;
use std::net::Ipv4Addr;
use tracing::{info, warn};

const TOTAL_HEARTBEATS: usize = 10000;
const BATCH_SIZE: usize = 1000;
const PROJECTS: [&str; 4] = ["Alpha", "Beta", "Gamma", "Delta"];
const LANGUAGES: [&str; 5] = ["Python", "JavaScript", "Go", "Rust", "C++"];
const FILE_EXTENSIONS: [&str; 5] = [".py", ".js", ".go", ".rs", ".cpp"];
const BRANCHES: [&str; 4] = ["main", "dev", "feature/x", "bugfix/y"];
const USER_AGENT: &str =
    "wakatime/v1.115.2 (linux-6.14.1) go1.24.2 vscode/1.100.0 vscode-wakatime/25.0.3";

struct HeartbeatParams<'a> {
    projects: &'a [&'a str],
    languages: &'a [&'a str],
    file_extensions: &'a [&'a str],
    branches: &'a [&'a str],
    ip_address: IpNetwork,
    user_agent: &'a str,
}

pub async fn seed_database(pool: &DbPool) -> Result<(), Box<dyn std::error::Error>> {
    info!("🔄 Starting database seeding...");

    let user = {
        let mut conn = pool.get()?;

        if let Some(existing_user) = User::find_by_github_id(&mut conn, -1)? {
            warn!(
                "⚠️  Dummy user already exists (id: {}), skipping seeding.",
                existing_user.id
            );
            return Ok(());
        }

        User::create_or_update(
            &mut conn,
            -1,
            "Test User",
            "https://avatars.githubusercontent.com/u/999999",
        )?
    };

    info!(
        "✅ Created dummy user: {} (API Key: {})",
        user.name, user.api_key
    );

    generate_random_heartbeats(pool, user.id, TOTAL_HEARTBEATS).await?;

    info!("✅ Database seeding completed successfully!");
    Ok(())
}

async fn generate_random_heartbeats(
    pool: &DbPool,
    user_id: i32,
    count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::rng();
    let ip_address = IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(127, 0, 0, 1), 32)?);

    info!("🔄 Generating {} random heartbeats...", count);

    let params = HeartbeatParams {
        projects: &PROJECTS,
        languages: &LANGUAGES,
        file_extensions: &FILE_EXTENSIONS,
        branches: &BRANCHES,
        ip_address,
        user_agent: USER_AGENT,
    };

    let mut batch = Vec::with_capacity(BATCH_SIZE);

    for _ in 0..count {
        let heartbeat = generate_random_heartbeat(&mut rng, user_id, &params);
        batch.push(heartbeat);

        if batch.len() == BATCH_SIZE {
            let current_batch = std::mem::take(&mut batch);
            store_heartbeats_in_db_count_only(pool, current_batch).await?;
        }
    }

    if !batch.is_empty() {
        let current_batch = std::mem::take(&mut batch);
        store_heartbeats_in_db_count_only(pool, current_batch).await?;
    }

    info!("✅ Inserted {} heartbeats into the database", count);

    Ok(())
}

fn generate_random_heartbeat<R: Rng>(
    rng: &mut R,
    user_id: i32,
    params: &HeartbeatParams,
) -> NewHeartbeat {
    let project = params.projects.choose(rng).unwrap();
    let language = params.languages.choose(rng).unwrap();
    let extension = params.file_extensions.choose(rng).unwrap();
    let branch = params.branches.choose(rng).unwrap();

    // generate random filename
    let filename: String = (0..8)
        .map(|_| rng.random_range(b'a'..=b'z') as char)
        .collect();

    let entity = format!(
        "/home/user/Documents/GitHub/{}/src/{}{}",
        project.to_lowercase(),
        filename,
        extension
    );

    // generate random timestamp within the last week
    let now = Utc::now();
    let week_in_seconds = 604800;
    let offset_seconds = rng.random_range(0..week_in_seconds);
    let time = now - chrono::Duration::seconds(offset_seconds);

    NewHeartbeat {
        user_id,
        time,
        entity,
        type_: "file".to_string(),
        ip_address: params.ip_address,
        project: Some(project.to_string()),
        branch: Some(branch.to_string()),
        language: Some(language.to_string()),
        category: Some("coding".to_string()),
        is_write: Some(rng.random_bool(0.5)),
        editor: Some("vscode".to_string()),
        operating_system: Some("linux".to_string()),
        machine: Some("test-machine".to_string()),
        user_agent: params.user_agent.to_string(),
        lines: Some(rng.random_range(1..=1000)),
        project_root_count: None,
        dependencies: None,
        line_additions: None,
        line_deletions: None,
        lineno: Some(rng.random_range(1..=100)),
        cursorpos: Some(rng.random_range(0..500)),
        source_type: Some(SourceType::Seeding as i16),
        project_id: None,
    }
}
