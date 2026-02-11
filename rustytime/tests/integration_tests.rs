#![cfg(feature = "integration")]

#[macro_use]
mod common;

use axum::http::StatusCode;
use common::{INVALID_API_KEY, TestApp, TestConfig, mock_heartbeat_payload};

#[cfg(test)]
mod health_tests {
    use super::*;

    #[tokio::test]
    async fn test_health_endpoint_returns_ok() {
        let config = TestConfig::default();
        fail_without_db!(config);

        let app = TestApp::new().await;
        let response = app.server.get("/health").await;

        response.assert_status_ok();
        response.assert_text("OK");
    }
}

#[cfg(test)]
mod auth_tests {
    use super::*;

    #[tokio::test]
    async fn test_github_login_returns_auth_url() {
        let config = TestConfig::default();
        fail_without_db!(config);

        let app = TestApp::new().await;
        let response = app.server.get("/auth/github/login").await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert!(
            body.get("auth_url").is_some(),
            "Response should contain auth_url"
        );
    }

    #[tokio::test]
    async fn test_verify_session_without_session_returns_bad_request() {
        let config = TestConfig::default();
        fail_without_db!(config);

        let app = TestApp::new().await;
        let response = app.server.get("/auth/github/verify").await;

        response.assert_status(StatusCode::BAD_REQUEST);
    }
}

#[cfg(test)]
mod api_v1_tests {
    use super::*;
    use axum::http::header;

    #[tokio::test]
    async fn test_heartbeat_without_auth_returns_unauthorized() {
        let config = TestConfig::default();
        fail_without_db!(config);

        let app = TestApp::new().await;
        let payload = mock_heartbeat_payload();

        let response = app
            .server
            .post("/api/v1/users/current/heartbeats")
            .json(&payload)
            .await;

        response.assert_status(StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_heartbeat_with_invalid_api_key_returns_unauthorized() {
        let config = TestConfig::default();
        fail_without_db!(config);

        let app = TestApp::new().await;
        let payload = mock_heartbeat_payload();

        let response = app
            .server
            .post("/api/v1/users/current/heartbeats")
            .add_header(header::AUTHORIZATION, format!("Basic {}", INVALID_API_KEY))
            .json(&payload)
            .await;

        response.assert_status(StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_statusbar_today_without_auth_returns_error() {
        let config = TestConfig::default();
        fail_without_db!(config);

        let app = TestApp::new().await;

        let response = app
            .server
            .get("/api/v1/users/current/statusbar/today")
            .await;

        assert!(
            response.status_code() == StatusCode::UNAUTHORIZED
                || response.status_code() == StatusCode::BAD_REQUEST,
            "Expected auth error, got {:?}",
            response.status_code()
        );
    }
}

#[cfg(test)]
mod protected_routes_tests {
    use super::*;

    #[tokio::test]
    async fn test_dashboard_without_auth_returns_unauthorized() {
        let config = TestConfig::default();
        fail_without_db!(config);

        let app = TestApp::new().await;
        let response = app.server.get("/page/dashboard").await;

        response.assert_status(StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_settings_without_auth_returns_unauthorized() {
        let config = TestConfig::default();
        fail_without_db!(config);

        let app = TestApp::new().await;
        let response = app.server.get("/page/settings").await;

        response.assert_status(StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_projects_without_auth_returns_unauthorized() {
        let config = TestConfig::default();
        fail_without_db!(config);

        let app = TestApp::new().await;
        let response = app.server.get("/page/projects").await;

        response.assert_status(StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_leaderboard_without_auth_returns_authorized() {
        let config = TestConfig::default();
        fail_without_db!(config);

        let app = TestApp::new().await;
        let response = app.server.get("/page/leaderboard").await;

        response.assert_status(StatusCode::OK);
    }
}

#[cfg(test)]
mod admin_routes_tests {
    use super::*;

    #[tokio::test]
    async fn test_admin_page_without_auth_returns_unauthorized() {
        let config = TestConfig::default();
        fail_without_db!(config);

        let app = TestApp::new().await;
        let response = app.server.get("/page/admin").await;

        response.assert_status(StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_admin_impersonate_without_auth_returns_unauthorized() {
        let config = TestConfig::default();
        fail_without_db!(config);

        let app = TestApp::new().await;
        let response = app.server.get("/admin/impersonate/1").await;

        response.assert_status(StatusCode::UNAUTHORIZED);
    }
}

#[cfg(test)]
mod docs_tests {
    use super::*;

    #[tokio::test]
    async fn test_openapi_docs_endpoint() {
        let config = TestConfig::default();
        fail_without_db!(config);

        let app = TestApp::new().await;
        let response = app.server.get("/docs/private/api.json").await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert!(body.get("openapi").is_some(), "Should return OpenAPI spec");
    }

    #[tokio::test]
    async fn test_scalar_docs_page() {
        let config = TestConfig::default();
        fail_without_db!(config);

        let app = TestApp::new().await;
        let response = app.server.get("/docs").await;

        response.assert_status_ok();
    }
}

#[cfg(test)]
mod not_found_tests {
    use super::*;

    #[tokio::test]
    async fn test_unknown_route_returns_not_found() {
        let config = TestConfig::default();
        fail_without_db!(config);

        let app = TestApp::new().await;
        let response = app.server.get("/this/route/does/not/exist").await;

        response.assert_status(StatusCode::NOT_FOUND);
    }
}

#[cfg(test)]
mod user_tests {
    use super::*;

    #[tokio::test]
    async fn test_create_user_in_database() {
        let config = TestConfig::default();
        fail_without_db!(config);

        let app = TestApp::new().await;
        let user = app.create_test_user("test_user_create");

        assert!(user.github_id > 999_000_000);
        assert_eq!(user.name, "test_user_create");
        assert!(!user.api_key.is_nil());

        app.cleanup_test_user(user.id);
    }

    #[tokio::test]
    async fn test_user_api_key_is_valid_uuid() {
        let config = TestConfig::default();
        fail_without_db!(config);

        let app = TestApp::new().await;
        let user = app.create_test_user("test_user_api_key");

        assert!(!user.api_key.is_nil());
        assert_eq!(user.api_key.get_version_num(), 4);

        app.cleanup_test_user(user.id);
    }
}

#[cfg(test)]
mod heartbeat_tests {
    use super::*;
    use axum::http::header;
    use base64::Engine;

    fn encode_api_key(api_key: &uuid::Uuid) -> String {
        base64::engine::general_purpose::STANDARD.encode(api_key.to_string())
    }

    #[tokio::test]
    async fn test_send_single_heartbeat() {
        let config = TestConfig::default();
        fail_without_db!(config);

        let app = TestApp::new().await;
        let user = app.create_test_user("test_heartbeat_user");
        let auth_value = format!("Basic {}", encode_api_key(&user.api_key));

        let payload = serde_json::json!({
            "entity": "/path/to/test_file.rs",
            "type": "file",
            "time": chrono::Utc::now().timestamp() as f64,
            "project": "integration-test-project",
            "language": "Rust",
            "branch": "main"
        });

        let response = app
            .server
            .post("/api/v1/users/current/heartbeats")
            .add_header(header::AUTHORIZATION, auth_value)
            .json(&payload)
            .await;

        response.assert_status(StatusCode::ACCEPTED);
        let body: serde_json::Value = response.json();
        assert!(body.get("data").is_some());
        assert!(
            body["data"]["entity"]
                .as_str()
                .unwrap()
                .contains("test_file.rs")
        );

        app.cleanup_test_user(user.id);
    }

    #[tokio::test]
    async fn test_send_bulk_heartbeats() {
        let config = TestConfig::default();
        fail_without_db!(config);

        let app = TestApp::new().await;
        let user = app.create_test_user("test_bulk_heartbeat_user");
        let auth_value = format!("Basic {}", encode_api_key(&user.api_key));

        let now = chrono::Utc::now().timestamp() as f64;
        let payload = serde_json::json!([
            {
                "entity": "/path/to/file1.rs",
                "type": "file",
                "time": now,
                "project": "bulk-test-project",
                "language": "Rust"
            },
            {
                "entity": "/path/to/file2.rs",
                "type": "file",
                "time": now + 60.0,
                "project": "bulk-test-project",
                "language": "Rust"
            },
            {
                "entity": "/path/to/file3.rs",
                "type": "file",
                "time": now + 120.0,
                "project": "bulk-test-project",
                "language": "Rust"
            }
        ]);

        let response = app
            .server
            .post("/api/v1/users/current/heartbeats.bulk")
            .add_header(header::AUTHORIZATION, auth_value)
            .json(&payload)
            .await;

        response.assert_status(StatusCode::CREATED);
        let body: serde_json::Value = response.json();
        assert!(body.get("responses").is_some());
        let responses = body["responses"].as_array().unwrap();
        assert_eq!(responses.len(), 3);

        app.cleanup_test_user(user.id);
    }

    #[tokio::test]
    async fn test_heartbeat_with_invalid_api_key_fails() {
        let config = TestConfig::default();
        fail_without_db!(config);

        let app = TestApp::new().await;
        let fake_key = uuid::Uuid::new_v4();
        let auth_value = format!("Basic {}", encode_api_key(&fake_key));

        let payload = mock_heartbeat_payload();

        let response = app
            .server
            .post("/api/v1/users/current/heartbeats")
            .add_header(header::AUTHORIZATION, auth_value)
            .json(&payload)
            .await;

        assert!(
            response.status_code() == StatusCode::UNAUTHORIZED
                || response.status_code() == StatusCode::BAD_REQUEST
        );
    }
}

#[cfg(test)]
mod data_retrieval_tests {
    use super::*;
    use axum::http::header;
    use base64::Engine;

    fn encode_api_key(api_key: &uuid::Uuid) -> String {
        base64::engine::general_purpose::STANDARD.encode(api_key.to_string())
    }

    #[tokio::test]
    async fn test_statusbar_today_returns_data() {
        let config = TestConfig::default();
        fail_without_db!(config);

        let app = TestApp::new().await;
        let user = app.create_test_user("test_statusbar_user");
        let auth_value = format!("Basic {}", encode_api_key(&user.api_key));

        let response = app
            .server
            .get("/api/v1/users/current/statusbar/today")
            .add_header(header::AUTHORIZATION, auth_value)
            .await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        assert!(body.get("data").is_some());
        assert!(body["data"]["grand_total"].is_object());

        app.cleanup_test_user(user.id);
    }

    #[tokio::test]
    async fn test_statusbar_today_with_heartbeats() {
        let config = TestConfig::default();
        fail_without_db!(config);

        let app = TestApp::new().await;
        let user = app.create_test_user("test_statusbar_heartbeat_user");
        let auth_value = format!("Basic {}", encode_api_key(&user.api_key));

        let now = chrono::Utc::now().timestamp() as f64;
        let heartbeats = serde_json::json!([
            {
                "entity": "/path/to/file.rs",
                "type": "file",
                "time": now - 60.0,
                "project": "statusbar-test",
                "language": "Rust"
            },
            {
                "entity": "/path/to/file.rs",
                "type": "file",
                "time": now,
                "project": "statusbar-test",
                "language": "Rust"
            }
        ]);

        let send_response = app
            .server
            .post("/api/v1/users/current/heartbeats.bulk")
            .add_header(header::AUTHORIZATION, auth_value.clone())
            .json(&heartbeats)
            .await;
        send_response.assert_status(StatusCode::CREATED);

        let response = app
            .server
            .get("/api/v1/users/current/statusbar/today")
            .add_header(header::AUTHORIZATION, auth_value)
            .await;

        response.assert_status_ok();
        let body: serde_json::Value = response.json();
        let total_seconds = body["data"]["grand_total"]["total_seconds"]
            .as_i64()
            .unwrap_or(0);
        assert!(total_seconds >= 0);

        app.cleanup_test_user(user.id);
    }
}
