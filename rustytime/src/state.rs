use crate::db::connection::DbPool;
use crate::jobs::import::ImportStore;
use crate::utils::cache::AppCache;
use crate::utils::metrics::MetricsTracker;
use oauth2::{EndpointNotSet, EndpointSet, basic::BasicClient};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::RwLock;

/// App state that holds shared resources
#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub github_client:
        BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>,
    pub http_client: reqwest::Client,
    pub metrics: MetricsTracker,
    pub import_store: Arc<RwLock<Option<ImportStore>>>,
    pub cache: AppCache,
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
            import_store: Arc::new(RwLock::new(None)),
            cache: AppCache::new(),
        }
    }

    pub fn set_import_store(&self, store: ImportStore) {
        let import_store = self.import_store.clone();
        tokio::spawn(async move {
            let mut guard = import_store.write().await;
            *guard = Some(store);
        });
    }
}
