use diesel::connection::{Instrumentation, InstrumentationEvent, set_default_instrumentation};
use std::time::Instant;
use tracing::{debug, info, warn};

struct QueryInstrumentation {
    query_start: Option<Instant>,
    query_count: u64,
    cache_hits: u64,
}

impl QueryInstrumentation {
    fn new() -> Self {
        Self {
            query_start: None,
            query_count: 0,
            cache_hits: 0,
        }
    }
}

impl Instrumentation for QueryInstrumentation {
    fn on_connection_event(&mut self, event: InstrumentationEvent<'_>) {
        match event {
            InstrumentationEvent::StartQuery { .. } => {
                self.query_start = Some(Instant::now());
                self.query_count += 1;
            }
            InstrumentationEvent::CacheQuery { sql, .. } => {
                self.cache_hits += 1;
                debug!(
                    sql = %sql,
                    cache_hits = self.cache_hits,
                    "Statement cache hit"
                );
            }
            InstrumentationEvent::FinishQuery { query, error, .. } => {
                let duration = self.query_start.take().map(|s| s.elapsed());
                let duration_ms = duration.map(|d| d.as_secs_f64() * 1000.0).unwrap_or(0.0);

                if let Some(err) = error {
                    warn!(
                        duration_ms = %format!("{duration_ms:.2}"),
                        error = %err,
                        query = %query,
                        query_count = self.query_count,
                        "SQL query failed"
                    );
                } else if duration_ms > 100.0 {
                    warn!(
                        duration_ms = %format!("{duration_ms:.2}"),
                        query = %query,
                        query_count = self.query_count,
                        cache_hits = self.cache_hits,
                        "Slow SQL query"
                    );
                } else {
                    debug!(
                        duration_ms = %format!("{duration_ms:.2}"),
                        query = %query,
                        query_count = self.query_count,
                        "SQL query executed"
                    );
                }
            }
            InstrumentationEvent::BeginTransaction { depth, .. } => {
                debug!(depth = %depth, "Transaction started");
            }
            InstrumentationEvent::CommitTransaction { depth, .. } => {
                debug!(depth = %depth, "Transaction committed");
            }
            InstrumentationEvent::RollbackTransaction { depth, .. } => {
                warn!(depth = %depth, "Transaction rolled back");
            }
            _ => {}
        }
    }
}

pub fn init_query_instrumentation() {
    set_default_instrumentation(|| Some(Box::new(QueryInstrumentation::new())))
        .expect("Failed to set default Diesel instrumentation");
    info!("✅ SQL query instrumentation enabled");
}
