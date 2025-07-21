use axum::extract::{ConnectInfo, Json, Path, State};
use axum::http::StatusCode;
use chrono::Utc;
use diesel::prelude::*;
use ipnetwork::IpNetwork;
use serde_json::json;
use std::net::SocketAddr;

use crate::db::DbPool;
use crate::models::heartbeat::*;
use crate::schema::heartbeats;
use crate::state::AppState;
use crate::utils::auth::{get_user_id_from_api_key, get_valid_api_key};

const HEARTBEAT_TIMEOUT: i32 = 120; // 2 minutes in seconds

/// Process heartbeat request and store in the database
async fn process_heartbeat_request(
    pool: &DbPool,
    id: String,
    addr: SocketAddr,
    headers: axum::http::HeaderMap,
    uri: axum::http::Uri,
    heartbeat_input: HeartbeatInput,
) -> Result<Json<HeartbeatApiResponseVariant>, StatusCode> {
    if id != "current" {
        return Err(StatusCode::BAD_REQUEST);
    }

    let heartbeat_requests = heartbeat_input.into_vec();
    if heartbeat_requests.len() > 25 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let api_key = get_valid_api_key(&headers, &uri).await;
    let api_key = match api_key {
        Some(key) => key,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let user_result = get_user_id_from_api_key(pool, &api_key).await;
    let user_id: i32 = match user_result {
        Some(id) => id,
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    let ip_network = IpNetwork::from(addr.ip());

    if heartbeat_requests.len() == 1 {
        let new_heartbeat = NewHeartbeat::from_request(
            heartbeat_requests.into_iter().next().unwrap(),
            user_id,
            ip_network,
        );

        match store_heartbeats_in_db(pool, vec![new_heartbeat]).await {
            Ok(mut heartbeats) => {
                if let Some(heartbeat) = heartbeats.pop() {
                    let response = HeartbeatApiResponse {
                        data: heartbeat.into(),
                    };
                    Ok(Json(HeartbeatApiResponseVariant::Single(response)))
                } else {
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
            Err(e) => {
                eprintln!("❌ Error inserting heartbeat: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    } else {
        let new_heartbeats: Vec<NewHeartbeat> = heartbeat_requests
            .into_iter()
            .map(|req| NewHeartbeat::from_request(req, user_id, ip_network))
            .collect();

        match store_heartbeats_in_db(pool, new_heartbeats).await {
            Ok(heartbeats) => {
                if heartbeats.is_empty() {
                    let response = HeartbeatsApiResponse { data: vec![] };
                    Ok(Json(HeartbeatApiResponseVariant::Multiple(response)))
                } else {
                    let response = HeartbeatsApiResponse {
                        data: heartbeats.into_iter().map(|h| h.into()).collect(),
                    };
                    Ok(Json(HeartbeatApiResponseVariant::Multiple(response)))
                }
            }
            Err(e) => {
                eprintln!("❌ Error inserting heartbeats: {}", e);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

/// Handler to create heartbeats
pub async fn create_heartbeats(
    State(app_state): State<AppState>,
    Path(id): Path<String>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: axum::http::HeaderMap,
    uri: axum::http::Uri,
    Json(heartbeat_input): Json<HeartbeatInput>,
) -> Result<Json<HeartbeatApiResponseVariant>, StatusCode> {
    process_heartbeat_request(&app_state.db_pool, id, addr, headers, uri, heartbeat_input).await
}

/// Handler to get today's status bar data
pub async fn get_statusbar_today(
    State(app_state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    uri: axum::http::Uri,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let user_id: i32 = if id != "current" {
        match id.parse::<i32>() {
            Ok(id) => id,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        }
    } else {
        let api_key = get_valid_api_key(&headers, &uri).await;
        let api_key = match api_key {
            Some(key) => key,
            None => return Err(StatusCode::UNAUTHORIZED),
        };

        let user_result = get_user_id_from_api_key(&app_state.db_pool, &api_key).await;
        match user_result {
            Some(id) => id,
            None => return Err(StatusCode::UNAUTHORIZED),
        }
    };

    match get_today_heartbeats(&app_state.db_pool, user_id).await {
        Ok(heartbeats) => {
            let total_seconds = calculate_duration_seconds(heartbeats);
            let hours = total_seconds / 3600;
            let minutes = (total_seconds % 3600) / 60;

            let digital_time = format!("{:02}:{:02}", hours, minutes);
            let text_time = if hours > 0 {
                format!("{} hrs {} mins", hours, minutes)
            } else {
                format!("{} mins", minutes)
            };

            Ok(Json(json!({
                "data": {
                    "grand_total": {
                        "decimal": format!("{:.2}", total_seconds as f64 / 3600.0),
                        "digital": digital_time,
                        "hours": hours,
                        "minutes": minutes,
                        "text": text_time,
                        "total_seconds": total_seconds
                    }
                }
            })))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Store heartbeats in the database
async fn store_heartbeats_in_db(
    pool: &DbPool,
    new_heartbeats: Vec<NewHeartbeat>,
) -> Result<Vec<Heartbeat>, diesel::result::Error> {
    let pool = pool.clone();
    tokio::task::spawn_blocking(move || {
        let mut connection = pool.get().expect("Failed to get DB connection from pool");

        match diesel::insert_into(heartbeats::table)
            .values(&new_heartbeats)
            .on_conflict_do_nothing()
            .get_results(&mut connection)
        {
            Ok(heartbeats) => Ok(heartbeats),
            Err(diesel::result::Error::NotFound) => Ok(vec![]),
            Err(e) => Err(e),
        }
    })
    .await
    .unwrap()
}

/// Get today's heartbeats for a user
async fn get_today_heartbeats(
    pool: &DbPool,
    user_id: i32,
) -> Result<Vec<Heartbeat>, diesel::result::Error> {
    let pool = pool.clone();
    tokio::task::spawn_blocking(move || {
        let mut connection = pool.get().expect("Failed to get DB connection from pool");
        let today = Utc::now().date_naive();
        let tomorrow = today.succ_opt().unwrap_or(today);

        heartbeats::table
            .filter(heartbeats::user_id.eq(user_id))
            .filter(heartbeats::created_at.ge(today.and_hms_opt(0, 0, 0).unwrap().and_utc()))
            .filter(heartbeats::created_at.lt(tomorrow.and_hms_opt(0, 0, 0).unwrap().and_utc()))
            .select(Heartbeat::as_select())
            .load(&mut connection)
    })
    .await
    .unwrap()
}

/// Calculate total duration in seconds for a list of heartbeats
fn calculate_duration_seconds(mut heartbeats: Vec<Heartbeat>) -> i32 {
    if heartbeats.len() < 2 {
        return 0;
    }

    // sort heartbeats by created_at in-place
    heartbeats.sort_unstable_by(|a, b| a.created_at.cmp(&b.created_at));

    heartbeats
        .windows(2)
        .map(|pair| {
            let diff_seconds = pair[1]
                .created_at
                .signed_duration_since(pair[0].created_at)
                .num_seconds() as i32;
            
            // only count positive differences within timeout
            diff_seconds.clamp(0, HEARTBEAT_TIMEOUT)
        })
        .sum()
}
