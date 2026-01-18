use axum::extract::ConnectInfo;
use axum::extract::{Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use diesel::prelude::*;
use ipnetwork::IpNetwork;
use serde_json::json;
use std::net::IpAddr;
use std::net::SocketAddr;

use crate::db::connection::DbPool;
use crate::get_db_conn;
use crate::models::heartbeat::Heartbeat;
use crate::models::heartbeat::*;
use crate::models::project::get_or_create_project_id;
use crate::schema::heartbeats;
use crate::state::AppState;
use crate::utils::auth::{get_user_id_from_api_key, get_valid_api_key};
use crate::utils::http::extract_client_ip_from_headers;
use crate::utils::time::{TimeFormat, human_readable_duration};
use std::collections::{HashMap, hash_map};

const MAX_HEARTBEATS_PER_REQUEST: usize = 100;
const HEARTBEAT_INSERT_BATCH_SIZE: usize = 1_000; // avoids hitting Postgres' 65k parameter limit

/// Process heartbeat request and store in the database
async fn process_heartbeat_request(
    app_state: &AppState,
    id: String,
    client_ip: IpAddr,
    headers: axum::http::HeaderMap,
    uri: axum::http::Uri,
    heartbeat_input: HeartbeatInput,
) -> Result<Response, Response> {
    if id != "current" {
        return Err((StatusCode::BAD_REQUEST, "Bad request").into_response());
    }

    let heartbeat_requests = heartbeat_input.into_vec();
    if heartbeat_requests.len() > MAX_HEARTBEATS_PER_REQUEST {
        return Err((StatusCode::BAD_REQUEST, "Bad request").into_response());
    }

    let api_key = get_valid_api_key(&headers, &uri).await;
    let api_key = match api_key {
        Some(key) => key,
        None => return Err((StatusCode::UNAUTHORIZED, "Unauthorized").into_response()),
    };

    let user_result = get_user_id_from_api_key(&app_state.db_pool, &api_key).await;
    let user_id: i32 = match user_result {
        Some(id) => id,
        None => return Err((StatusCode::UNAUTHORIZED, "Unauthorized").into_response()),
    };

    let ip_network = IpNetwork::from(client_ip);

    if heartbeat_requests.len() == 1 {
        let new_heartbeat = NewHeartbeat::from_request(
            heartbeat_requests.into_iter().next().unwrap(),
            user_id,
            ip_network,
            &headers,
        );

        match store_heartbeats_in_db(&app_state.db_pool, vec![new_heartbeat]).await {
            Ok(mut stored_results) => {
                if let Some(heartbeat) = stored_results.pop() {
                    app_state.cache.invalidate_user_dashboard(user_id);
                    let response = HeartbeatApiResponse { data: heartbeat };
                    let response_data = Json(HeartbeatApiResponseVariant::Single(response));
                    Ok((StatusCode::ACCEPTED, response_data).into_response())
                } else {
                    Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
                        .into_response())
                }
            }
            Err(e) => {
                eprintln!("❌ Error inserting heartbeat: {}", e);
                Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response())
            }
        }
    } else {
        let new_heartbeats: Vec<NewHeartbeat> = heartbeat_requests
            .into_iter()
            .map(|req| NewHeartbeat::from_request(req, user_id, ip_network, &headers))
            .collect();

        match store_heartbeats_in_db(&app_state.db_pool, new_heartbeats).await {
            Ok(stored_results) => {
                if stored_results.is_empty() {
                    let response_data = Json(HeartbeatApiResponseVariant::Multiple(
                        HeartbeatBulkApiResponse { responses: vec![] },
                    ));
                    Ok((StatusCode::CREATED, response_data).into_response())
                } else {
                    app_state.cache.invalidate_user_dashboard(user_id);
                    let response_data = Json(HeartbeatApiResponseVariant::Multiple(
                        HeartbeatBulkApiResponse {
                            responses: stored_results
                                .into_iter()
                                .map(|heartbeat| BulkResponseItem(heartbeat, 201))
                                .collect(),
                        },
                    ));
                    Ok((StatusCode::CREATED, response_data).into_response())
                }
            }
            Err(e) => {
                eprintln!("❌ Error inserting heartbeats: {}", e);
                Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response())
            }
        }
    }
}

/// Handler to create heartbeats, trusting Cloudflare headers when enabled.
pub async fn create_heartbeats(
    State(app_state): State<AppState>,
    Path(id): Path<String>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    headers: axum::http::HeaderMap,
    uri: axum::http::Uri,
    Json(heartbeat_input): Json<HeartbeatInput>,
) -> Result<Response, Response> {
    let client_ip = extract_client_ip_from_headers(&headers, addr);
    process_heartbeat_request(&app_state, id, client_ip, headers, uri, heartbeat_input).await
}

/// Handler to get today's status bar data
pub async fn get_statusbar_today(
    State(app_state): State<AppState>,
    Path(id): Path<String>,
    headers: axum::http::HeaderMap,
    uri: axum::http::Uri,
) -> Result<Json<serde_json::Value>, Response> {
    let user_id: i32 = if id != "current" {
        match id.parse::<i32>() {
            Ok(id) => id,
            Err(_) => return Err((StatusCode::BAD_REQUEST, "Bad request").into_response()),
        }
    } else {
        let api_key = get_valid_api_key(&headers, &uri).await;
        let api_key = match api_key {
            Some(key) => key,
            None => return Err((StatusCode::BAD_REQUEST, "Bad request").into_response()),
        };

        let user_result = get_user_id_from_api_key(&app_state.db_pool, &api_key).await;
        match user_result {
            Some(id) => id,
            None => return Err((StatusCode::BAD_REQUEST, "Bad request").into_response()),
        }
    };

    // calculate today's date range
    let today = Utc::now().date_naive();
    let start_of_day = today.and_hms_opt(0, 0, 0).unwrap().and_utc();
    let end_of_day = today
        .succ_opt()
        .unwrap_or(today)
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();

    let mut conn = get_db_conn!(app_state);

    match Heartbeat::get_user_duration_seconds(
        &mut conn,
        DurationInput {
            user_id: Some(user_id),
            start_date: Some(start_of_day),
            end_date: Some(end_of_day),
            project: None,
            entity: None,
            language: None,
            type_filter: None,
        },
    ) {
        Ok(total_seconds) => {
            let time_obj = human_readable_duration(total_seconds, TimeFormat::HourMinute);

            Ok(Json(json!({
                "data": {
                    "grand_total": {
                        "decimal": format!("{:.2}", total_seconds as f64 / 3600.0),
                        "digital": format!("{:02}:{:02}", time_obj.hours, time_obj.minutes),
                        "hours": time_obj.hours,
                        "minutes": time_obj.minutes,
                        "text": time_obj.human_readable,
                        "total_seconds": total_seconds
                    }
                }
            })))
        }
        Err(err) => {
            eprintln!("❌ Error calculating duration: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response())
        }
    }
}

/// Store heartbeats in the database and return them
pub async fn store_heartbeats_in_db(
    pool: &DbPool,
    new_heartbeats: Vec<NewHeartbeat>,
) -> Result<Vec<HeartbeatResponse>, diesel::result::Error> {
    store_heartbeats_in_db_internal(pool, new_heartbeats, true)
        .await
        .map(|(responses, _)| responses)
}

/// Store heartbeats and report how many unique entries were inserted
pub async fn store_heartbeats_in_db_count_only(
    pool: &DbPool,
    new_heartbeats: Vec<NewHeartbeat>,
) -> Result<usize, diesel::result::Error> {
    store_heartbeats_in_db_internal(pool, new_heartbeats, false)
        .await
        .map(|(_, inserted)| inserted)
}

async fn store_heartbeats_in_db_internal(
    pool: &DbPool,
    new_heartbeats: Vec<NewHeartbeat>,
    include_responses: bool,
) -> Result<(Vec<HeartbeatResponse>, usize), diesel::result::Error> {
    let pool = pool.clone();

    tokio::task::spawn_blocking(move || {
        let mut connection = pool.get().expect("Failed to get DB connection from pool");

        connection.transaction(|conn| {
            let mut heartbeat_keys = if include_responses {
                Some(Vec::with_capacity(new_heartbeats.len()))
            } else {
                None
            };
            let mut seen: HashMap<(i32, chrono::DateTime<Utc>), ()> =
                HashMap::with_capacity(new_heartbeats.len());
            let mut deduplicated = Vec::with_capacity(new_heartbeats.len());

            for mut heartbeat in new_heartbeats {
                if heartbeat.project_id.is_none()
                    && let Some(project_name) = heartbeat.project.as_ref()
                {
                    let project_id =
                        get_or_create_project_id(conn, heartbeat.user_id, project_name, None)?;
                    heartbeat.project_id = Some(project_id);
                }

                let key = (heartbeat.user_id, heartbeat.time);
                if let Some(keys) = heartbeat_keys.as_mut() {
                    keys.push(key);
                }

                if let hash_map::Entry::Vacant(e) = seen.entry(key) {
                    e.insert(());
                    deduplicated.push(heartbeat);
                }
            }

            let mut inserted_total = 0usize;
            for chunk in deduplicated.chunks(HEARTBEAT_INSERT_BATCH_SIZE) {
                inserted_total += diesel::insert_into(heartbeats::table)
                    .values(chunk)
                    .on_conflict((heartbeats::user_id, heartbeats::time))
                    .do_nothing()
                    .execute(conn)?;
            }

            let responses = if include_responses {
                let unique_keys: Vec<_> = seen.keys().copied().collect();

                let mut heartbeat_cache: HashMap<(i32, chrono::DateTime<Utc>), Heartbeat> =
                    HashMap::new();
                for (uid, t) in unique_keys {
                    let hb = heartbeats::table
                        .filter(heartbeats::user_id.eq(uid))
                        .filter(heartbeats::time.eq(t))
                        .first::<Heartbeat>(conn)?;
                    heartbeat_cache.insert((uid, t), hb);
                }

                heartbeat_keys
                    .unwrap()
                    .into_iter()
                    .map(|key| {
                        let hb = heartbeat_cache.get(&key).unwrap().clone();
                        HeartbeatResponse::from(hb)
                    })
                    .collect()
            } else {
                Vec::new()
            };

            Ok((responses, inserted_total))
        })
    })
    .await
    .unwrap()
}
