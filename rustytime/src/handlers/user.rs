use axum::extract::{ConnectInfo, Json, Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use diesel::prelude::*;
use ipnetwork::IpNetwork;
use serde_json::json;
use std::net::SocketAddr;

use crate::db::DbPool;
use crate::models::heartbeat::Heartbeat;
use crate::models::heartbeat::*;
use crate::schema::heartbeats;
use crate::state::AppState;
use crate::utils::auth::{get_user_id_from_api_key, get_valid_api_key};
use crate::utils::time::human_readable_duration;

/// Process heartbeat request and store in the database
async fn process_heartbeat_request(
    pool: &DbPool,
    id: String,
    addr: SocketAddr,
    headers: axum::http::HeaderMap,
    uri: axum::http::Uri,
    heartbeat_input: HeartbeatInput,
) -> Result<Response, Response> {
    if id != "current" {
        return Err((StatusCode::BAD_REQUEST, "Bad request").into_response());
    }

    let heartbeat_requests = heartbeat_input.into_vec();
    if heartbeat_requests.len() > 25 {
        return Err((StatusCode::BAD_REQUEST, "Bad request").into_response());
    }

    let api_key = get_valid_api_key(&headers, &uri).await;
    let api_key = match api_key {
        Some(key) => key,
        None => return Err((StatusCode::UNAUTHORIZED, "Unauthorized").into_response()),
    };

    let user_result = get_user_id_from_api_key(pool, &api_key).await;
    let user_id: i32 = match user_result {
        Some(id) => id,
        None => return Err((StatusCode::UNAUTHORIZED, "Unauthorized").into_response()),
    };

    let ip_network = IpNetwork::from(addr.ip());

    if heartbeat_requests.len() == 1 {
        let new_heartbeat = NewHeartbeat::from_request(
            heartbeat_requests.into_iter().next().unwrap(),
            user_id,
            ip_network,
            &headers,
        );

        match store_heartbeats_in_db(pool, vec![new_heartbeat]).await {
            Ok(mut heartbeats) => {
                if let Some(heartbeat) = heartbeats.pop() {
                    let response = HeartbeatApiResponse {
                        data: heartbeat.into(),
                    };
                    Ok((
                        StatusCode::CREATED,
                        Json(HeartbeatApiResponseVariant::Single(response)),
                    )
                        .into_response())
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

        match store_heartbeats_in_db(pool, new_heartbeats).await {
            Ok(heartbeats) => {
                if heartbeats.is_empty() {
                    let response = HeartbeatsApiResponse { data: vec![] };
                    Ok((
                        StatusCode::CREATED,
                        Json(HeartbeatApiResponseVariant::Multiple(response)),
                    )
                        .into_response())
                } else {
                    let response = HeartbeatsApiResponse {
                        data: heartbeats.into_iter().map(|h| h.into()).collect(),
                    };
                    Ok((
                        StatusCode::CREATED,
                        Json(HeartbeatApiResponseVariant::Multiple(response)),
                    )
                        .into_response())
                }
            }
            Err(e) => {
                eprintln!("❌ Error inserting heartbeats: {}", e);
                Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response())
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
) -> Result<Response, Response> {
    process_heartbeat_request(&app_state.db_pool, id, addr, headers, uri, heartbeat_input).await
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

    let mut conn = app_state.db_pool.get().map_err(|err| {
        eprintln!("Database connection error: {}", err);
        (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
    })?;

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
            let time_obj = human_readable_duration(total_seconds);

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
        Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()),
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

        connection.transaction(|conn| {
            match diesel::insert_into(heartbeats::table)
                .values(&new_heartbeats)
                .on_conflict_do_nothing()
                .get_results(conn)
            {
                Ok(heartbeats) => Ok(heartbeats),
                Err(diesel::result::Error::NotFound) => Ok(vec![]),
                Err(e) => Err(e),
            }
        })
    })
    .await
    .unwrap()
}
