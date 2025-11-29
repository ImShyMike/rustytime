use aide::openapi::{
    ApiKeyLocation, Components, License, OpenApi, ReferenceOr, SecurityScheme, Server, Tag,
};
use indexmap::IndexMap;

use crate::utils::session::SESSION_COOKIE_NAME;

pub fn get_openapi_docs() -> OpenApi {
    let mut openapi = OpenApi::default();

    openapi.info.title = "rustytime API".to_string();
    openapi.info.summary = Some("Self-hosted, WakaTime-compatible time tracking backend.".into());
    openapi.info.version = env!("CARGO_PKG_VERSION").to_string();
    openapi.info.description = Some("Blazingly fast time tracking for developers!".to_string());
    openapi.info.license = Some(License {
        name: "GNU Affero General Public License v3.0".into(),
        identifier: Some("AGPL-3.0-only".into()),
        ..Default::default()
    });

    openapi.servers = vec![
        Server {
            url: "https://api-rustytime.shymike.dev".into(),
            description: Some("Production deployment".into()),
            ..Default::default()
        },
        Server {
            url: "http://localhost:3000".into(),
            description: Some("Local development".into()),
            ..Default::default()
        },
    ];

    openapi.tags = vec![
        Tag {
            name: "Authentication".into(),
            description: Some("OAuth login, logout, and session verification.".into()),
            ..Default::default()
        },
        Tag {
            name: "Pages".into(),
            description: Some("Main data endpoints for every page on the frontend.".into()),
            ..Default::default()
        },
        Tag {
            name: "Data".into(),
            description: Some("CRUD endpoints.".into()),
            ..Default::default()
        },
        Tag {
            name: "Admin".into(),
            description: Some("Admin only CRUD endpoints.".into()),
            ..Default::default()
        },
        Tag {
            name: "WakaTime Compatibility".into(),
            description: Some(
                "API key protected endpoints that are compatible with WakaTime.".into(),
            ),
            ..Default::default()
        },
        Tag {
            name: "Health".into(),
            description: Some("Backend health checks.".into()),
            ..Default::default()
        },
        Tag {
            name: "Metrics".into(),
            description: Some("Prometheus metrics.".into()),
            ..Default::default()
        },
    ];

    let mut security_schemes = IndexMap::new();
    security_schemes.insert(
        "Authenticated".into(),
        ReferenceOr::Item(SecurityScheme::ApiKey {
            location: ApiKeyLocation::Cookie,
            name: SESSION_COOKIE_NAME.to_string(),
            description: Some("Session cookie issued after a successful GitHub OAuth flow.".into()),
            extensions: IndexMap::new(),
        }),
    );
    security_schemes.insert(
        "ApiKey".into(),
        ReferenceOr::Item(SecurityScheme::ApiKey {
            location: ApiKeyLocation::Header,
            name: "Authorization".into(),
            description: Some(
                "Use `Bearer <api_key>` or `Basic <base64_api_key>`` auth. A fallback `api_key` query parameter is also accepted.".into(),
            ),
            extensions: IndexMap::new(),
        }),
    );

    openapi.components = Some(Components {
        security_schemes,
        ..Default::default()
    });

    openapi
}
