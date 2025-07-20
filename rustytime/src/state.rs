use crate::db::DbPool;
use crate::utils::templates::TemplateEngine;
use oauth2::{EndpointNotSet, EndpointSet, basic::BasicClient};

/// App state that holds shared resources
#[derive(Clone)]
pub struct AppState {
    pub db_pool: DbPool,
    pub github_client:
        BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>,
    pub template_engine: TemplateEngine,
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
        template_engine: TemplateEngine,
    ) -> Self {
        Self {
            db_pool,
            github_client,
            template_engine,
        }
    }
}
