use crate::db::connection::DbPool;
use crate::utils::metrics::MetricsTracker;
use linguist::container::InMemoryLanguageContainer;
use oauth2::{EndpointNotSet, EndpointSet, basic::BasicClient};
use reqwest::Client;
use std::sync::Arc;

/// App state that holds shared resources
#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub github_client:
        BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>,
    pub http_client: reqwest::Client,
    pub metrics: MetricsTracker,
    pub language_container: Arc<InMemoryLanguageContainer>,
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
            language_container: Arc::new(InMemoryLanguageContainer::default()),
        }
    }
}
