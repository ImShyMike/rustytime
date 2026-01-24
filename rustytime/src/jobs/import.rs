use std::time::Duration;

use apalis::{
    layers::{WorkerBuilderExt, prometheus::PrometheusLayer},
    prelude::{
        BackoffConfig, BoxDynError, Codec, Data, IntervalStrategy, StrategyBuilder, TaskSink,
        WorkerBuilder,
    },
};
use apalis_postgres::PostgresStorage;
use chrono::{DateTime, Duration as ChronoDuration, NaiveDate, SecondsFormat, Utc};
use futures::{FutureExt, TryFutureExt};
use reqwest::Client;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, from_str};
use sqlx::PgPool;
use tokio::signal::ctrl_c;
use tracing::{debug, error, info, info_span, warn};

use crate::db::connection::DbPool;
use crate::handlers::api::user::store_heartbeats_in_db_count_only;
use crate::models::heartbeat::{HackatimeHeartbeat, NewHeartbeat};
use crate::models::import_job::ImportJob as ImportJobModel;
use crate::utils::time::{determine_range, format_rfc3339, split_range_midpoint};

const HACKATIME_HEARTBEATS_ENDPOINT: &str = "https://hackatime.hackclub.com/api/v1/my/heartbeats";
const HEARTBEAT_IMPORT_BATCH_SIZE: usize = 1_000;
const HACKATIME_BODY_LOG_LIMIT: usize = 2_048;
const MINIMUM_HACKATIME_RANGE: ChronoDuration = ChronoDuration::hours(6);
const MAX_RANGE_SPLIT_DEPTH: u8 = 6;
const CUTOFF_YEAR: i32 = 2024;
const CUTOFF_MONTH_DAY: [u32; 2] = [1, 1];

#[derive(Clone)]
pub struct JsonCodec;

impl<T: Serialize + DeserializeOwned> Codec<T> for JsonCodec {
    type Compact = Vec<u8>;
    type Error = serde_json::Error;

    fn encode(input: &T) -> Result<Vec<u8>, Self::Error> {
        serde_json::to_vec(input)
    }

    fn decode(compact: &Vec<u8>) -> Result<T, Self::Error> {
        serde_json::from_slice(compact)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImportJob {
    pub user_id: i32,
    pub api_key: String,
    pub job_id: i64,
}

impl ImportJob {
    pub fn new(user_id: i32, api_key: String, job_id: i64) -> Self {
        Self {
            user_id,
            api_key,
            job_id,
        }
    }
}

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

#[derive(Deserialize)]
struct HackatimeHeartbeatResponse {
    heartbeats: Vec<HackatimeHeartbeat>,
}

struct ParsedHackatimeHeartbeats {
    heartbeats: Vec<HackatimeHeartbeat>,
    skipped: usize,
    salvaged: bool,
}

enum HackatimeFetchError {
    Unrecoverable(String),
    Recoverable {
        error: serde_json::Error,
        preview: String,
        body_length: usize,
    },
}

async fn run_import(job: ImportJob, pool: Data<DbPool>) -> Result<String, BoxDynError> {
    let started = std::time::Instant::now();
    let user_id = job.user_id;
    let job_id = job.job_id;

    let span = info_span!("import_job", user_id = user_id, job_id = job_id);
    let _guard = span.enter();

    info!("Starting import job");

    let http_client = Client::new();

    let cutoff = NaiveDate::from_ymd_opt(CUTOFF_YEAR, CUTOFF_MONTH_DAY[0], CUTOFF_MONTH_DAY[1])
        .expect("valid cutoff date")
        .and_hms_opt(0, 0, 0)
        .expect("valid cutoff time")
        .and_utc();

    let result = execute_import(&http_client, &pool, user_id, &job.api_key, cutoff).await;

    let elapsed = started.elapsed();

    let conn = &mut *pool.get().expect("Failed to get DB connection from pool");

    match result {
        Ok((imported, processed, requests, earliest_requested)) => {
            let start_date = earliest_requested
                .map(format_rfc3339)
                .unwrap_or_else(|| cutoff.to_rfc3339_opts(SecondsFormat::Millis, true));

            if let Err(e) = ImportJobModel::complete(
                conn,
                job_id,
                imported as i64,
                processed as i64,
                requests as i32,
                start_date.clone(),
                elapsed.as_secs_f64(),
            ) {
                error!(error = ?e, "Failed to update import job as completed");
            }

            info!(
                imported,
                processed,
                requests,
                elapsed_secs = elapsed.as_secs_f64(),
                "Import job completed successfully"
            );

            Ok(format!(
                "Import completed: {} imported, {} processed in {:.2}s",
                imported,
                processed,
                elapsed.as_secs_f64()
            ))
        }
        Err(error_message) => {
            if let Err(e) = ImportJobModel::fail(conn, job_id, &error_message) {
                error!(error = ?e, "Failed to update import job as failed");
            }

            error!(error = %error_message, "Import job failed");
            Err(error_message.into())
        }
    }
}

async fn execute_import(
    http_client: &Client,
    db_pool: &DbPool,
    user_id: i32,
    api_key: &str,
    cutoff: DateTime<Utc>,
) -> Result<(usize, usize, usize, Option<DateTime<Utc>>), String> {
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
            fetch_hackatime_heartbeats(http_client, api_key, range_start, period_end).await?;
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
                    match persist_heartbeat_chunk(db_pool, &mut chunked_heartbeats).await {
                        Ok(inserted) => total_inserted += inserted,
                        Err(err) => {
                            error!("Failed to persist imported heartbeats: {err}");
                            return Err("Failed to store imported heartbeats".to_string());
                        }
                    }
                }
            }

            if !chunked_heartbeats.is_empty() {
                match persist_heartbeat_chunk(db_pool, &mut chunked_heartbeats).await {
                    Ok(inserted) => total_inserted += inserted,
                    Err(err) => {
                        error!("Failed to persist imported heartbeats: {err}");
                        return Err("Failed to store imported heartbeats".to_string());
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

    Ok((
        total_inserted,
        total_processed,
        requests_made,
        earliest_requested,
    ))
}

async fn fetch_hackatime_heartbeats(
    client: &Client,
    api_key: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<HackatimeFetchResult, String> {
    fetch_hackatime_range(client, api_key, start, end, 0).await
}

async fn fetch_hackatime_range(
    client: &Client,
    api_key: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    depth: u8,
) -> Result<HackatimeFetchResult, String> {
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
            Err(HackatimeFetchError::Unrecoverable(msg)) => return Err(msg),
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
                    return Err("Failed to parse Hackatime response".to_string());
                }

                let Some(midpoint) = split_range_midpoint(range_start, range_end) else {
                    error!(
                        start = %range_start,
                        end = %range_end,
                        range_hours = range.num_hours(),
                        "Unable to split Hackatime range after parse error"
                    );
                    return Err("Failed to parse Hackatime response".to_string());
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
    client: &Client,
    api_key: &str,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
) -> Result<Vec<HackatimeHeartbeat>, HackatimeFetchError> {
    use reqwest::StatusCode;

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
            HackatimeFetchError::Unrecoverable("Failed to reach the Hackatime API".to_string())
        })?;

    let status = response.status();
    if status == StatusCode::UNAUTHORIZED {
        return Err(HackatimeFetchError::Unrecoverable(
            "Hackatime API key is invalid".to_string(),
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
                "Hackatime API responded with an error".to_string(),
            ));
        }
    }

    let raw_body = response.text().await.map_err(|err| {
        error!("Failed to read Hackatime response body: {err}");
        HackatimeFetchError::Unrecoverable("Failed to read Hackatime response body".to_string())
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

pub type ImportStore = PostgresStorage<ImportJob, Vec<u8>, JsonCodec, apalis_postgres::PgNotify>;

pub async fn create_storage(sqlx_pool: &PgPool) -> ImportStore {
    let storage_config = apalis_postgres::Config::new("import_jobs").with_poll_interval(
        StrategyBuilder::new()
            .apply(
                IntervalStrategy::new(Duration::from_secs(5))
                    .with_backoff(BackoffConfig::default()),
            )
            .build(),
    );

    PostgresStorage::new_with_notify(sqlx_pool, &storage_config).with_codec::<JsonCodec>()
}

pub async fn enqueue_import(
    storage: &ImportStore,
    user_id: i32,
    api_key: String,
    job_id: i64,
) -> Result<(), BoxDynError> {
    let job = ImportJob::new(user_id, api_key, job_id);
    let mut storage = storage.clone();
    storage.push(job).await?;
    Ok(())
}

pub async fn setup(
    sqlx_pool: PgPool,
    diesel_pool: DbPool,
) -> impl std::future::Future<Output = ()> {
    let import_store = create_storage(&sqlx_pool).await;

    WorkerBuilder::new("import-worker")
        .backend(import_store)
        .enable_tracing()
        .layer(PrometheusLayer::default())
        .catch_panic()
        .concurrency(2)
        .data(diesel_pool)
        .build(run_import)
        .run_until(ctrl_c())
        .map_err(|e| tracing::error!("Import worker error: {}", e))
        .map(|_| ())
}
