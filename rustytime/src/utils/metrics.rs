use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Request metrics structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    pub requests_per_second: f64,
}

impl Default for RequestMetrics {
    fn default() -> Self {
        Self {
            requests_per_second: 0.0,
        }
    }
}

/// Simple request metrics tracker
#[derive(Debug, Clone)]
pub struct MetricsTracker {
    /// Request timestamps for the last minute
    recent_requests: Arc<Mutex<VecDeque<Instant>>>,
}

impl Default for MetricsTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsTracker {
    /// Create a new metrics tracker
    pub fn new() -> Self {
        Self {
            recent_requests: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Log a new request
    pub fn record_request(&self) {
        let now = Instant::now();
        let mut recent_requests = self.recent_requests.lock().unwrap();

        // add the new request timestamp
        recent_requests.push_back(now);

        // remove requests older than 60 seconds
        let sixty_seconds_ago = now - Duration::from_secs(60);
        while let Some(&front_time) = recent_requests.front() {
            if front_time < sixty_seconds_ago {
                recent_requests.pop_front();
            } else {
                break;
            }
        }
    }

    /// Get current requests per second
    pub fn get_metrics(&self) -> RequestMetrics {
        let recent_requests = self.recent_requests.lock().unwrap();
        RequestMetrics {
            requests_per_second: recent_requests.len() as f64 / 60.0,
        }
    }
}
