#![cfg(feature = "seed")]

use crate::models::heartbeat::NewHeartbeat;
use crate::models::user::User;
use crate::schema::heartbeats;
use chrono::Utc;
use diesel::prelude::*;
use ipnetwork::{IpNetwork, Ipv4Network};
use rand::Rng;
use rand::prelude::IndexedRandom;
use std::net::Ipv4Addr;
use tracing::info;

const TOTAL_HEARTBEATS: usize = 10000;
const BATCH_SIZE: usize = 1000;
const PROJECTS: [&str; 4] = ["Alpha", "Beta", "Gamma", "Delta"];
const LANGUAGES: [&str; 5] = ["python", "javascript", "go", "rust", "c++"];
const FILE_EXTENSIONS: [&str; 5] = [".py", ".js", ".go", ".rs", ".cpp"];
const BRANCHES: [&str; 4] = ["main", "dev", "feature/x", "bugfix/y"];

pub fn seed_database(conn: &mut PgConnection) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting database seeding...");

    let Ok(user) = create_dummy_user(conn) else {
        info!("Dummy user already exists, skipping seeding.");
        return Ok(());
    };
    info!(
        "Created dummy user: {} (API Key: {})",
        user.name, user.api_key
    );

    generate_random_heartbeats(conn, user.id, TOTAL_HEARTBEATS)?;

    info!("Database seeding completed successfully!");
    Ok(())
}

fn create_dummy_user(conn: &mut PgConnection) -> Result<User, Box<dyn std::error::Error>> {
    if User::find_by_github_id(conn, -1)?.is_some() {
        return Err("Dummy user already exists".into());
    }

    let user = User::create_or_update(
        conn,
        -1,
        "Test User",
        "https://avatars.githubusercontent.com/u/999999",
    )?;

    Ok(user)
}

fn generate_random_heartbeats(
    conn: &mut PgConnection,
    user_id: i32,
    count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::rng();

    let ip_address = IpNetwork::V4(Ipv4Network::new(Ipv4Addr::new(127, 0, 0, 1), 32)?);
    let user_agent =
        "wakatime/v1.115.2 (linux-6.14.1) go1.24.2 vscode/1.100.0 vscode-wakatime/25.0.3";

    info!("Generating {} random heartbeats...", count);

    let mut heartbeats = Vec::with_capacity(count);
    for _ in 0..count {
        let heartbeat = generate_random_heartbeat(
            &mut rng,
            user_id,
            &PROJECTS,
            &LANGUAGES,
            &FILE_EXTENSIONS,
            &BRANCHES,
            ip_address,
            user_agent,
        );
        heartbeats.push(heartbeat);
    }

    for batch in heartbeats.chunks(BATCH_SIZE) {
        diesel::insert_into(heartbeats::table)
            .values(batch)
            .execute(conn)?;
    }

    Ok(())
}

fn generate_random_heartbeat<R: Rng>(
    rng: &mut R,
    user_id: i32,
    projects: &[&str],
    languages: &[&str],
    file_extensions: &[&str],
    branches: &[&str],
    ip_address: IpNetwork,
    user_agent: &str,
) -> NewHeartbeat {
    let project = projects.choose(rng).unwrap();
    let language = languages.choose(rng).unwrap();
    let extension = file_extensions.choose(rng).unwrap();
    let branch = branches.choose(rng).unwrap();

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
        ip_address,
        project: Some(project.to_string()),
        branch: Some(branch.to_string()),
        language: Some(language.to_string()),
        category: Some("coding".to_string()),
        is_write: Some(rng.random_bool(0.5)),
        editor: Some("vscode".to_string()),
        operating_system: Some("linux".to_string()),
        machine: "test-machine".to_string(),
        user_agent: user_agent.to_string(),
        lines: Some(rng.random_range(1..=1000)),
        project_root_count: None,
        dependencies: None,
        line_additions: None,
        line_deletions: None,
        lineno: Some(rng.random_range(1..=100)),
        cursorpos: Some(rng.random_range(0..500)),
    }
}
