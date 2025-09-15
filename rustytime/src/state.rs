use crate::db::DbPool;
use crate::utils::metrics::MetricsTracker;
use oauth2::{EndpointNotSet, EndpointSet, basic::BasicClient};
use reqwest::Client;

/// App state that holds shared resources
#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub github_client:
        BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>,
    pub http_client: reqwest::Client,
    pub metrics: MetricsTracker,
}

impl AppState {
    pub fn new(
        db_pool: DbPool,
        github_client: BasicClient<
            EndpointSet,
            EndpointNotSet,
            EndpointNotSet,
            EndpointNotSet,
            EndpointSet,
        >,
    ) -> Self {
        Self {
            db_pool,
            github_client,
            http_client: Client::new(),
            metrics: MetricsTracker::new(),
        }
    }
}
