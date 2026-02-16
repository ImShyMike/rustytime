use diesel::QueryResult;
use std::time::Instant;
use tracing::{debug, warn};

const SLOW_QUERY_THRESHOLD_MS: f64 = 100.0;

/// Execute a query that returns a `Vec<T>` and log the row count and duration.
pub fn load<T, F>(label: &str, f: F) -> QueryResult<Vec<T>>
where
    F: FnOnce() -> QueryResult<Vec<T>>,
{
    let start = Instant::now();
    let result = f();
    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

    match &result {
        Ok(rows) => {
            if duration_ms > SLOW_QUERY_THRESHOLD_MS {
                warn!(
                    label,
                    duration_ms = %format!("{duration_ms:.2}"),
                    rows_returned = rows.len(),
                    "Slow query"
                );
            } else {
                debug!(
                    label,
                    duration_ms = %format!("{duration_ms:.2}"),
                    rows_returned = rows.len(),
                    "Query loaded"
                );
            }
        }
        Err(err) => {
            warn!(
                label,
                duration_ms = %format!("{duration_ms:.2}"),
                error = %err,
                "Query load failed"
            );
        }
    }

    result
}

/// Execute a query that returns a single `T` and log the duration.
pub fn first<T, F>(label: &str, f: F) -> QueryResult<T>
where
    F: FnOnce() -> QueryResult<T>,
{
    let start = Instant::now();
    let result = f();
    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

    match &result {
        Ok(_) => {
            if duration_ms > SLOW_QUERY_THRESHOLD_MS {
                warn!(
                    label,
                    duration_ms = %format!("{duration_ms:.2}"),
                    rows_returned = 1,
                    "Slow query"
                );
            } else {
                debug!(
                    label,
                    duration_ms = %format!("{duration_ms:.2}"),
                    rows_returned = 1,
                    "Query first"
                );
            }
        }
        Err(err) => {
            warn!(
                label,
                duration_ms = %format!("{duration_ms:.2}"),
                rows_returned = 0,
                error = %err,
                "Query first failed"
            );
        }
    }

    result
}

/// Execute a statement (INSERT/UPDATE/DELETE) and log the affected row count.
pub fn execute<F>(label: &str, f: F) -> QueryResult<usize>
where
    F: FnOnce() -> QueryResult<usize>,
{
    let start = Instant::now();
    let result = f();
    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;

    match &result {
        Ok(affected) => {
            if duration_ms > SLOW_QUERY_THRESHOLD_MS {
                warn!(
                    label,
                    duration_ms = %format!("{duration_ms:.2}"),
                    rows_affected = affected,
                    "Slow execute"
                );
            } else {
                debug!(
                    label,
                    duration_ms = %format!("{duration_ms:.2}"),
                    rows_affected = affected,
                    "Query executed"
                );
            }
        }
        Err(err) => {
            warn!(
                label,
                duration_ms = %format!("{duration_ms:.2}"),
                error = %err,
                "Query execute failed"
            );
        }
    }

    result
}
