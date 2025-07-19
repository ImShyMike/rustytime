use axum::extract::Query;
use base64::prelude::*;
use diesel::prelude::*;
use serde::Deserialize;

use crate::db::DbPool;
use crate::schema::users::dsl;

async fn get_api_key_from_header(headers: &axum::http::HeaderMap) -> Option<String> {
    // parse "Authorization" header
    if let Some(auth_header) = headers.get("Authorization") {
        if let Ok(auth_str) = auth_header.to_str() {
            // check if it starts with "Bearer" or "Basic"
            if auth_str.starts_with("Bearer ") {
                // extract the API key
                let api_key = &auth_str[7..];
                return Some(api_key.to_string());
            } else if auth_str.starts_with("Basic ") {
                // decode base64
                let base64_key = &auth_str[6..];
                if let Ok(decoded) = BASE64_STANDARD.decode(base64_key) {
                    if let Ok(api_key) = String::from_utf8(decoded) {
                        return Some(api_key);
                    }
                }
            }
        }
    }
    None
}

async fn get_api_key_from_query(query: &axum::http::Uri) -> Option<String> {
    #[derive(Deserialize)]
    struct QueryParams {
        api_key: Option<String>,
    }

    if let Ok(params) = Query::<QueryParams>::try_from_uri(&query) {
        return params.0.api_key;
    }
    None
}

async fn validate_api_key(api_key: &str) -> bool {
    if api_key.len() != 36 {
        return false; // invalid length
    }
    if let Ok(_) = uuid::Uuid::parse_str(api_key) {
        return true;
    } else {
        return false; // invalid UUID format
    }
}

pub async fn get_valid_api_key(
    headers: &axum::http::HeaderMap,
    query: &axum::http::Uri,
) -> Option<String> {
    if let Some(api_key) = get_api_key_from_header(headers).await {
        if validate_api_key(&api_key).await {
            return Some(api_key);
        }
    } else if let Some(api_key) = get_api_key_from_query(query).await {
        if validate_api_key(&api_key).await {
            return Some(api_key);
        }
    }
    None
}

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
