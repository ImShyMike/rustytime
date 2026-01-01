use aide::NoApi;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::{Extension, Json};
use chrono::Datelike;
use chrono::{DateTime, Duration, NaiveDate, SecondsFormat, Utc};
use schemars::JsonSchema;
use serde::Deserialize;
use serde::Serialize;
use serde_json::{Value, from_str};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};
use tower_cookies::Cookies;
use tracing::{debug, error, info, info_span, warn};

use crate::db::connection::DbPool;
use crate::db_query;
use crate::handlers::api::user::store_heartbeats_in_db_count_only;
use crate::models::heartbeat::{HackatimeHeartbeat, NewHeartbeat};
use crate::models::user::User;
use crate::state::AppState;
use crate::utils::session::SessionManager;

const HACKATIME_HEARTBEATS_ENDPOINT: &str = "https://hackatime.hackclub.com/api/v1/my/heartbeats";
const HEARTBEAT_IMPORT_BATCH_SIZE: usize = 1_000;
const HACKATIME_BODY_LOG_LIMIT: usize = 2_048;
const MINIMUM_HACKATIME_RANGE: Duration = Duration::hours(6);
const MAX_RANGE_SPLIT_DEPTH: u8 = 6;
const CUTOFF_YEAR: i32 = 2024;
const CUTOFF_MONTH_DAY: [u32; 2] = [1, 1];

struct HackatimeFetchResult {
    heartbeats: Vec<HackatimeHeartbeat>,
    requests: usize,
}

impl HackatimeFetchResult {
    fn empty() -> Self {
        Self {
            heartbeats: Vec::new(),
            requests: 0,
        }
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct ImportQuery {
    api_key: String,
}

#[derive(Deserialize)]
struct HackatimeHeartbeatResponse {
    heartbeats: Vec<HackatimeHeartbeat>,
}

#[derive(Serialize, JsonSchema)]
pub struct ImportResponse {
    imported: usize,
    processed: usize,
    requests: usize,
    start_date: String,
    time_taken: f64,
}

struct ImportRunGuard {
    user_id: i32,
    locks: Arc<Mutex<HashSet<i32>>>,
}

impl ImportRunGuard {
    async fn acquire(locks: Arc<Mutex<HashSet<i32>>>, user_id: i32) -> Result<Self, Response> {
        let mut guard = locks.lock().await;
        if !guard.insert(user_id) {
            return Err((
                StatusCode::CONFLICT,
                "An import is already running for this user",
            )
                .into_response());
        }

        drop(guard);
        Ok(Self { user_id, locks })
    }
}

impl Drop for ImportRunGuard {
    fn drop(&mut self) {
        let locks = self.locks.clone();
        let user_id = self.user_id;
        tokio::spawn(async move {
            let mut guard = locks.lock().await;
            guard.remove(&user_id);
        });
    }
}

/// Handler for importing heartbeats via API
pub async fn import_heartbeats(
    State(app_state): State<AppState>,
    Query(query): Query<ImportQuery>,
    cookies: NoApi<Cookies>,
    user: NoApi<Option<Extension<User>>>,
) -> Result<Json<ImportResponse>, Response> {
    // get current user
    let current_user = user
        .0
        .expect("User should be authenticated since middleware validated authentication")
        .0;

    let Some(session_id) = SessionManager::get_session_from_cookies(&cookies) else {
        return Err((StatusCode::UNAUTHORIZED, "User session is invalid").into_response());
    };

    let Some(session_data) = db_query!(
        SessionManager::validate_session(&app_state.db_pool, session_id).await,
        "Session validation error"
    ) else {
        return Err((StatusCode::UNAUTHORIZED, "User session is invalid").into_response());
    };

    if session_data.impersonated_by.is_some() && !current_user.is_owner() {
        return Err((
            StatusCode::FORBIDDEN,
            "Impersonators cannot perform data imports",
        )
            .into_response());
    }

    let api_key = query.api_key.trim().to_string();
    if api_key.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "api_key query parameter is required",
        )
            .into_response());
    }

    let user_id = current_user.id;
    let (result_tx, result_rx) = oneshot::channel();
    let background_state = app_state.clone();

    // detach the import process to run in the background
    tokio::spawn(async move {
        let result = run_hackatime_import(background_state, user_id, api_key).await;
        if result_tx.send(result).is_err() {
            info!(
                user_id = user_id,
                "Hackatime import completed after client disconnected"
            );
        }
    });

    match result_rx.await {
        Ok(Ok(response)) => Ok(Json(response)),
        Ok(Err(response)) => Err(response),
        Err(_) => {
            error!(
                user_id = user_id,
                "Hackatime import task ended before reporting result"
            );
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "Hackatime import task ended unexpectedly",
            )
                .into_response())
        }
    }
}

async fn run_hackatime_import(
    app_state: AppState,
    user_id: i32,
    api_key: String,
) -> Result<ImportResponse, Response> {
    let http_client = app_state.http_client.clone();
    let db_pool = app_state.db_pool.clone();

    // acquire import lock for user
    let _import_guard = ImportRunGuard::acquire(app_state.import_locks.clone(), user_id).await?;

    let cutoff = NaiveDate::from_ymd_opt(CUTOFF_YEAR, CUTOFF_MONTH_DAY[0], CUTOFF_MONTH_DAY[1])
        .expect("valid cutoff date")
        .and_hms_opt(0, 0, 0)
        .expect("valid cutoff time")
        .and_utc();

    let span = info_span!("import_heartbeats", user_id = user_id);
    let _guard = span.enter();
    info!(cutoff = %cutoff, "Starting Hackatime import");

    let timer = std::time::Instant::now();

    let mut period_end = Utc::now();
    let mut total_processed = 0usize;
    let mut total_inserted = 0usize;
    let mut requests_made = 0usize;
    let mut earliest_requested: Option<DateTime<Utc>> = None;

    while period_end > cutoff {
        let (range_start, next_period_end) = determine_range(period_end, cutoff);
        if range_start >= period_end {
            break;
        }

        debug!(start = %range_start, end = %period_end, "Requesting Hackatime heartbeats");

        let fetch_result =
            fetch_hackatime_heartbeats(&http_client, &api_key, range_start, period_end).await?;
        requests_made += fetch_result.requests;
        earliest_requested = Some(range_start);
        let heartbeats = fetch_result.heartbeats;

        if !heartbeats.is_empty() {
            info!(
                user_id = user_id,
                start = %range_start,
                end = %period_end,
                count = heartbeats.len(),
                "Fetched Hackatime heartbeats"
            );

            total_processed += heartbeats.len();

            let mut chunked_heartbeats = Vec::with_capacity(HEARTBEAT_IMPORT_BATCH_SIZE);
            for hb in heartbeats {
                chunked_heartbeats.push(hb.to_new_heartbeat(user_id));
                if chunked_heartbeats.len() == HEARTBEAT_IMPORT_BATCH_SIZE {
                    match persist_heartbeat_chunk(&db_pool, &mut chunked_heartbeats).await {
                        Ok(inserted) => total_inserted += inserted,
                        Err(err) => {
                            error!("Failed to persist imported heartbeats: {err}");
                            return Err((
                                StatusCode::INTERNAL_SERVER_ERROR,
                                "Failed to store imported heartbeats",
                            )
                                .into_response());
                        }
                    }
                }
            }

            if !chunked_heartbeats.is_empty() {
                match persist_heartbeat_chunk(&db_pool, &mut chunked_heartbeats).await {
                    Ok(inserted) => total_inserted += inserted,
                    Err(err) => {
                        error!("Failed to persist imported heartbeats: {err}");
                        return Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            "Failed to store imported heartbeats",
                        )
                            .into_response());
                    }
                }
            }
        } else {
            debug!(start = %range_start, end = %period_end, "No heartbeats for range");
        }

        if next_period_end <= cutoff {
            break;
        }

        period_end = next_period_end;
    }

    info!(
        imported = total_inserted,
        processed = total_processed,
        requests = requests_made,
        "Hackatime import finished"
    );

    let time_taken = timer.elapsed().as_secs_f64();

    Ok(ImportResponse {
        imported: total_inserted,
        processed: total_processed,
        requests: requests_made,
        start_date: earliest_requested
            .map(format_rfc3339)
            .unwrap_or_else(|| cutoff.to_rfc3339_opts(SecondsFormat::Millis, true)),
        time_taken,
    })
}

fn determine_range(
    period_end: DateTime<Utc>,
    cutoff: DateTime<Utc>,
) -> (DateTime<Utc>, DateTime<Utc>) {
    let adjusted_end = period_end - Duration::nanoseconds(1);
    let month_start = adjusted_end
        .date_naive()
        .with_day(1)
        .expect("every month has a first day")
        .and_hms_opt(0, 0, 0)
        .expect("valid start of month")
        .and_utc();

    let range_start = if month_start > cutoff {
        month_start
    } else {
        cutoff
    };
    (range_start, month_start)
}

fn split_range_midpoint(start: DateTime<Utc>, end: DateTime<Utc>) -> Option<DateTime<Utc>> {
    if end <= start {
        return None;
    }

    let duration = end - start;
    let half = duration / 2;
    if half <= Duration::zero() {
        return None;
    }

    let midpoint = start.checked_add_signed(half)?;
    if midpoint <= start || midpoint >= end {
        None
    } else {
        Some(midpoint)
    }
}

enum HackatimeFetchError {
    Unrecoverable(Response),
    Recoverable {
        error: serde_json::Error,
        preview: String,
        body_length: usize,
    },
}

async fn fetch_hackatime_heartbeats(
    client: &reqwest::Client,
    api_key: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<HackatimeFetchResult, Response> {
    fetch_hackatime_range(client, api_key, start, end, 0).await
}

async fn fetch_hackatime_range(
    client: &reqwest::Client,
    api_key: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    depth: u8,
) -> Result<HackatimeFetchResult, Response> {
    if end <= start {
        return Ok(HackatimeFetchResult::empty());
    }

    debug!(
        start = %start,
        end = %end,
        range_hours = (end - start).num_hours(),
        split_depth = depth,
        "Fetching Hackatime heartbeats for range"
    );

    let mut stack = vec![(start, end, depth)];
    let mut aggregated = HackatimeFetchResult::empty();

    while let Some((range_start, range_end, current_depth)) = stack.pop() {
        if range_end <= range_start {
            continue;
        }

        let fetch_outcome = fetch_hackatime_once(client, api_key, range_start, range_end).await;
        aggregated.requests += 1;
        match fetch_outcome {
            Ok(mut heartbeats) => {
                aggregated.heartbeats.append(&mut heartbeats);
            }
            Err(HackatimeFetchError::Unrecoverable(response)) => return Err(response),
            Err(HackatimeFetchError::Recoverable {
                error,
                preview,
                body_length,
            }) => {
                let range = range_end - range_start;
                if range <= MINIMUM_HACKATIME_RANGE || current_depth >= MAX_RANGE_SPLIT_DEPTH {
                    error!(
                        error = %error,
                        body_preview = %preview,
                        body_length,
                        range_hours = range.num_hours(),
                        split_depth = current_depth,
                        "Failed to parse Hackatime response despite range splitting"
                    );
                    return Err((
                        StatusCode::BAD_GATEWAY,
                        "Failed to parse Hackatime response",
                    )
                        .into_response());
                }

                let Some(midpoint) = split_range_midpoint(range_start, range_end) else {
                    error!(
                        start = %range_start,
                        end = %range_end,
                        range_hours = range.num_hours(),
                        "Unable to split Hackatime range after parse error"
                    );
                    return Err((
                        StatusCode::BAD_GATEWAY,
                        "Failed to parse Hackatime response",
                    )
                        .into_response());
                };

                warn!(
                    start = %range_start,
                    end = %range_end,
                    midpoint = %midpoint,
                    range_hours = range.num_hours(),
                    split_depth = current_depth,
                    error = %error,
                    "Hackatime response too large; retrying with smaller window"
                );

                stack.push((midpoint, range_end, current_depth + 1));
                stack.push((range_start, midpoint, current_depth + 1));
            }
        }
    }

    Ok(aggregated)
}

async fn fetch_hackatime_once(
    client: &reqwest::Client,
    api_key: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<HackatimeHeartbeat>, HackatimeFetchError> {
    if end <= start {
        return Ok(vec![]);
    }

    let response = client
        .get(HACKATIME_HEARTBEATS_ENDPOINT)
        .bearer_auth(api_key)
        .query(&[
            ("start_time", format_rfc3339(start)),
            ("end_time", format_rfc3339(end)),
        ])
        .send()
        .await
        .map_err(|err| {
            error!("Failed to reach the Hackatime API: {err}");
            HackatimeFetchError::Unrecoverable(
                (StatusCode::BAD_GATEWAY, "Failed to reach the Hackatime API").into_response(),
            )
        })?;

    let status = response.status();
    if status == StatusCode::UNAUTHORIZED {
        return Err(HackatimeFetchError::Unrecoverable(
            (StatusCode::UNAUTHORIZED, "Hackatime API key is invalid").into_response(),
        ));
    }

    if !status.is_success() {
        if status.as_u16() == 524 {
            warn!("Hackatime API request timed out (524)");
            return Err(HackatimeFetchError::Recoverable {
                error: serde_json::Error::io(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    "Cloudflare timeout (524)",
                )),
                preview: String::from("Cloudflare timeout (524)"),
                body_length: 0,
            });
        } else {
            error!("Hackatime API returned status {}", status);
            return Err(HackatimeFetchError::Unrecoverable(
                (
                    StatusCode::BAD_GATEWAY,
                    "Hackatime API responded with an error",
                )
                    .into_response(),
            ));
        }
    }

    let raw_body = response.text().await.map_err(|err| {
        error!("Failed to read Hackatime response body: {err}");
        HackatimeFetchError::Unrecoverable(
            (
                StatusCode::BAD_GATEWAY,
                "Failed to read Hackatime response body",
            )
                .into_response(),
        )
    })?;

    match parse_hackatime_body(&raw_body) {
        Ok(parsed) => {
            if parsed.salvaged {
                warn!(
                    skipped = parsed.skipped,
                    returned = parsed.heartbeats.len(),
                    "Hackatime response had malformed heartbeats that were skipped"
                );
            }
            Ok(parsed.heartbeats)
        }
        Err(error) => {
            let preview = raw_body
                .chars()
                .take(HACKATIME_BODY_LOG_LIMIT)
                .collect::<String>();
            Err(HackatimeFetchError::Recoverable {
                error,
                preview,
                body_length: raw_body.len(),
            })
        }
    }
}

struct ParsedHackatimeHeartbeats {
    heartbeats: Vec<HackatimeHeartbeat>,
    skipped: usize,
    salvaged: bool,
}

fn parse_hackatime_body(body: &str) -> Result<ParsedHackatimeHeartbeats, serde_json::Error> {
    match from_str::<HackatimeHeartbeatResponse>(body) {
        Ok(payload) => Ok(ParsedHackatimeHeartbeats {
            heartbeats: payload.heartbeats,
            skipped: 0,
            salvaged: false,
        }),
        Err(primary_err) => {
            let value: Value = serde_json::from_str(body)?;
            let Some(array) = value.get("heartbeats").and_then(|v| v.as_array()) else {
                return Err(primary_err);
            };

            let mut recovered = Vec::with_capacity(array.len());
            let mut skipped = 0usize;

            for (idx, hb_value) in array.iter().enumerate() {
                match serde_json::from_value::<HackatimeHeartbeat>(hb_value.clone()) {
                    Ok(heartbeat) => recovered.push(heartbeat),
                    Err(err) => {
                        skipped += 1;
                        let preview = hb_value
                            .to_string()
                            .chars()
                            .take(HACKATIME_BODY_LOG_LIMIT)
                            .collect::<String>();
                        debug!(
                            index = idx,
                            error = %err,
                            heartbeat_preview = %preview,
                            "Skipping malformed Hackatime heartbeat"
                        );
                    }
                }
            }

            if recovered.is_empty() {
                Err(primary_err)
            } else {
                Ok(ParsedHackatimeHeartbeats {
                    heartbeats: recovered,
                    skipped,
                    salvaged: true,
                })
            }
        }
    }
}

#[inline(always)]
fn format_rfc3339(time: DateTime<Utc>) -> String {
    time.to_rfc3339_opts(SecondsFormat::Millis, true)
}

async fn persist_heartbeat_chunk(
    pool: &DbPool,
    buffer: &mut Vec<NewHeartbeat>,
) -> Result<usize, diesel::result::Error> {
    if buffer.is_empty() {
        return Ok(0);
    }

    let chunk = std::mem::take(buffer);
    store_heartbeats_in_db_count_only(pool, chunk).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tokio::time::{Duration, sleep};

    #[test]
    fn hackatime_response() {
        let payload = json!({
            "start_time": "2024-01-01T00:00:00Z",
            "end_time": "2024-01-02T00:00:00Z",
            "total_seconds": 1234.567,
            "heartbeats": [{
                "id": 1,
                "user_id": 1,
                "entity": "main.rs",
                "type": "file",
                "time": 1_700_000_000.0
            }]
        });

        let parsed: HackatimeHeartbeatResponse = serde_json::from_value(payload)
            .expect("response should deserialize even without all fields");

        assert_eq!(parsed.heartbeats.len(), 1);
        assert_eq!(parsed.heartbeats[0].entity, "main.rs");
    }

    #[test]
    fn hackatime_response_salvages_malformed_entries() {
        let payload = json!({
            "heartbeats": [
                {
                    "id": 1,
                    "user_id": 1,
                    "entity": "good.rs",
                    "type": "file",
                    "time": 1_700_000_000.0
                },
                {
                    "id": 2,
                    "user_id": 1,
                    "entity": "bad.rs",
                    "type": "file",
                    "time": null
                }
            ]
        })
        .to_string();

        let parsed = parse_hackatime_body(&payload).expect("should salvage valid heartbeats");
        assert!(parsed.salvaged);
        assert_eq!(parsed.skipped, 1);
        assert_eq!(parsed.heartbeats.len(), 1);
        assert_eq!(parsed.heartbeats[0].entity, "good.rs");
    }

    #[test]
    fn hackatime_response_errors_when_nothing_salvageable() {
        let payload = json!({
            "heartbeats": [
                {"foo": "bar"}
            ]
        })
        .to_string();

        assert!(parse_hackatime_body(&payload).is_err());
    }

    #[test]
    fn split_range_midpoint_finds_center() {
        let start = DateTime::from_timestamp(1_700_000_000, 0).expect("valid start");
        let end = start + chrono::Duration::days(30);
        let midpoint = split_range_midpoint(start, end).expect("midpoint should exist");
        assert!(midpoint > start);
        assert!(midpoint < end);
        let delta_start = midpoint - start;
        let delta_end = end - midpoint;
        assert!((delta_start.num_seconds() - delta_end.num_seconds()).abs() <= 1);
    }

    #[test]
    fn split_range_midpoint_rejects_equal_bounds() {
        let start = DateTime::from_timestamp(1_700_000_000, 0).expect("valid start");
        assert!(split_range_midpoint(start, start).is_none());
    }

    #[tokio::test]
    async fn import_guard_prevents_parallel_imports() {
        let locks = Arc::new(Mutex::new(HashSet::new()));
        let guard = ImportRunGuard::acquire(locks.clone(), 42)
            .await
            .expect("first guard acquisition should succeed");

        let Err(response) = ImportRunGuard::acquire(locks.clone(), 42).await else {
            panic!("second acquisition should fail");
        };
        assert_eq!(response.status(), StatusCode::CONFLICT);

        drop(guard);
        sleep(Duration::from_millis(5)).await;

        ImportRunGuard::acquire(locks, 42)
            .await
            .expect("guard should be released after drop");
    }
}
