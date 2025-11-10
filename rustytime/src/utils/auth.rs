use axum::extract::Query;
use base64::prelude::*;
use diesel::prelude::*;
use serde::Deserialize;

use crate::db::connection::DbPool;
use crate::schema::users::dsl;

/// Try to get API key from the "Authorization" header
fn get_api_key_from_header(headers: &axum::http::HeaderMap) -> Option<String> {
    // parse "Authorization" header
    if let Some(auth_header) = headers.get("Authorization")
        && let Ok(auth_str) = auth_header.to_str()
    {
        // check if it starts with "Bearer" or "Basic"
        if let Some(api_key) = auth_str.strip_prefix("Bearer ") {
            return Some(api_key.to_string());
        } else if let Some(base64_key) = auth_str.strip_prefix("Basic ") {
            // decode base64
            if let Ok(decoded) = BASE64_STANDARD.decode(base64_key)
                && let Ok(api_key) = String::from_utf8(decoded)
            {
                return Some(api_key);
            }
        }
    }
    None
}

#[derive(Deserialize)]
struct QueryParams {
    api_key: Option<String>,
}

/// Try to get API key from the URI's query parameters
fn get_api_key_from_query(query: &axum::http::Uri) -> Option<String> {
    if let Ok(params) = Query::<QueryParams>::try_from_uri(query) {
        return params.0.api_key;
    }
    None
}

/// Check if the API key is a valid UUID with dashes
fn validate_api_key(api_key: &str) -> bool {
    api_key.len() == 36 && uuid::Uuid::parse_str(api_key).is_ok()
}

// Tries to get a valid API key from headers or query parameters in that order
pub async fn get_valid_api_key(
    headers: &axum::http::HeaderMap,
    query: &axum::http::Uri,
) -> Option<String> {
    if let Some(api_key) = get_api_key_from_header(headers) {
        if validate_api_key(&api_key) {
            return Some(api_key);
        }
    } else if let Some(api_key) = get_api_key_from_query(query)
        && validate_api_key(&api_key)
    {
        return Some(api_key);
    }
    None
}

/// Get user ID from the API key
pub async fn get_user_id_from_api_key(pool: &DbPool, api_key_value: &str) -> Option<i32> {
    let api_key_uuid = uuid::Uuid::parse_str(api_key_value).ok()?;
    let mut conn = pool.get().ok()?;
    let user_id: i32 = dsl::users
        .filter(dsl::api_key.eq(api_key_uuid))
        .select(dsl::id)
        .first(&mut conn)
        .ok()?;
    Some(user_id)
}
