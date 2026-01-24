use super::*;

// =============================================================================
// get_api_key_from_header tests
// =============================================================================

#[test]
fn extracts_bearer_token_from_authorization_header() {
    let headers = {
        let mut map = axum::http::HeaderMap::new();
        map.insert(
            "Authorization",
            "Bearer 123e4567-e89b-12d3-a456-426614174000"
                .parse()
                .unwrap(),
        );
        map
    };
    let api_key = get_api_key_from_header(&headers).unwrap();
    assert_eq!(api_key, "123e4567-e89b-12d3-a456-426614174000".to_string());
}

#[test]
fn extracts_api_key_from_basic_authorization_header() {
    let headers = {
        let mut map = axum::http::HeaderMap::new();
        let basic_value = BASE64_STANDARD.encode("123e4567-e89b-12d3-a456-426614174000");
        map.insert(
            "Authorization",
            format!("Basic {}", basic_value).parse().unwrap(),
        );
        map
    };
    let api_key = get_api_key_from_header(&headers).unwrap();
    assert_eq!(api_key, "123e4567-e89b-12d3-a456-426614174000".to_string());
}

#[test]
fn missing_authorization_header_returns_none() {
    let headers = axum::http::HeaderMap::new();
    assert!(get_api_key_from_header(&headers).is_none());
}

#[test]
fn invalid_base64_in_basic_auth_returns_none() {
    let headers = {
        let mut map = axum::http::HeaderMap::new();
        map.insert(
            "Authorization",
            "Basic !!!invalid-base64!!!".parse().unwrap(),
        );
        map
    };
    assert!(get_api_key_from_header(&headers).is_none());
}

#[test]
fn invalid_utf8_after_base64_decode_returns_none() {
    let headers = {
        let mut map = axum::http::HeaderMap::new();
        let invalid_utf8 = BASE64_STANDARD.encode([0x80, 0x81, 0x82, 0x83]);
        map.insert(
            "Authorization",
            format!("Basic {}", invalid_utf8).parse().unwrap(),
        );
        map
    };
    assert!(get_api_key_from_header(&headers).is_none());
}

#[test]
fn empty_bearer_token_returns_empty_string() {
    let headers = {
        let mut map = axum::http::HeaderMap::new();
        map.insert("Authorization", "Bearer ".parse().unwrap());
        map
    };
    let api_key = get_api_key_from_header(&headers).unwrap();
    assert_eq!(api_key, "");
}

#[test]
fn bearer_prefix_is_case_sensitive() {
    let headers = {
        let mut map = axum::http::HeaderMap::new();
        map.insert(
            "Authorization",
            "bearer 123e4567-e89b-12d3-a456-426614174000"
                .parse()
                .unwrap(),
        );
        map
    };
    assert!(get_api_key_from_header(&headers).is_none());
}

#[test]
fn basic_prefix_is_case_sensitive() {
    let headers = {
        let mut map = axum::http::HeaderMap::new();
        let basic_value = BASE64_STANDARD.encode("123e4567-e89b-12d3-a456-426614174000");
        map.insert(
            "Authorization",
            format!("basic {}", basic_value).parse().unwrap(),
        );
        map
    };
    assert!(get_api_key_from_header(&headers).is_none());
}

#[test]
fn bearer_token_strips_whitespaces() {
    let headers = {
        let mut map = axum::http::HeaderMap::new();
        map.insert(
            "Authorization",
            "Bearer  token-with-leading-space".parse().unwrap(),
        );
        map
    };
    let api_key = get_api_key_from_header(&headers).unwrap();
    assert_eq!(api_key, "token-with-leading-space");
}

#[test]
fn bearer_without_space_returns_none() {
    let headers = {
        let mut map = axum::http::HeaderMap::new();
        map.insert(
            "Authorization",
            "Bearer123e4567-e89b-12d3-a456-426614174000"
                .parse()
                .unwrap(),
        );
        map
    };
    assert!(get_api_key_from_header(&headers).is_none());
}

#[test]
fn unrecognized_auth_scheme_returns_none() {
    let headers = {
        let mut map = axum::http::HeaderMap::new();
        map.insert(
            "Authorization",
            "Digest 123e4567-e89b-12d3-a456-426614174000"
                .parse()
                .unwrap(),
        );
        map
    };
    assert!(get_api_key_from_header(&headers).is_none());
}

// =============================================================================
// get_api_key_from_query tests
// =============================================================================

#[test]
fn extracts_api_key_from_query_parameter() {
    let uri: axum::http::Uri = "/path?api_key=123e4567-e89b-12d3-a456-426614174000"
        .parse()
        .unwrap();
    let api_key = get_api_key_from_query(&uri).unwrap();
    assert_eq!(api_key, "123e4567-e89b-12d3-a456-426614174000".to_string());
}

#[test]
fn missing_api_key_query_parameter_returns_none() {
    let uri: axum::http::Uri = "/path?other_param=value".parse().unwrap();
    assert!(get_api_key_from_query(&uri).is_none());
}

#[test]
fn empty_api_key_query_parameter_returns_empty_string() {
    let uri: axum::http::Uri = "/path?api_key=".parse().unwrap();
    let api_key = get_api_key_from_query(&uri).unwrap();
    assert_eq!(api_key, "");
}

#[test]
fn api_key_extracted_with_multiple_query_parameters() {
    let uri: axum::http::Uri = "/path?foo=bar&api_key=123e4567-e89b-12d3-a456-426614174000&baz=qux"
        .parse()
        .unwrap();
    let api_key = get_api_key_from_query(&uri).unwrap();
    assert_eq!(api_key, "123e4567-e89b-12d3-a456-426614174000");
}

#[test]
fn no_query_string_returns_none() {
    let uri: axum::http::Uri = "/path".parse().unwrap();
    assert!(get_api_key_from_query(&uri).is_none());
}

// =============================================================================
// validate_api_key tests
// =============================================================================

#[test]
fn validates_api_key_format_using_uuid_semantics() {
    let valid_key = "123e4567-e89b-12d3-a456-426614174000";
    let invalid_key = "invalid-api-key";
    assert!(validate_api_key(valid_key));
    assert!(!validate_api_key(invalid_key));
}

#[test]
fn api_key_too_short_fails_validation() {
    let short_key = "123e4567-e89b-12d3-a456";
    assert!(!validate_api_key(short_key));
}

#[test]
fn api_key_too_long_fails_validation() {
    let long_key = "123e4567-e89b-12d3-a456-426614174000-extra";
    assert!(!validate_api_key(long_key));
}

#[test]
fn correct_length_but_invalid_uuid_format_fails() {
    let invalid_uuid = "not-a-valid-uuid-format-here1234";
    assert_eq!(invalid_uuid.len(), 32);
    assert!(!validate_api_key(invalid_uuid));

    let invalid_36 = "gggggggg-gggg-gggg-gggg-gggggggggggg";
    assert_eq!(invalid_36.len(), 36);
    assert!(!validate_api_key(invalid_36));
}

#[test]
fn uuid_without_dashes_fails_validation() {
    let no_dashes = "123e4567e89b12d3a456426614174000";
    assert_eq!(no_dashes.len(), 32);
    assert!(!validate_api_key(no_dashes));
}

#[test]
fn uuid_v1_is_valid() {
    let uuid_v1 = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";
    assert!(validate_api_key(uuid_v1));
}

#[test]
fn uuid_v4_is_valid() {
    let uuid_v4 = "550e8400-e29b-41d4-a716-446655440000";
    assert!(validate_api_key(uuid_v4));
}

#[test]
fn nil_uuid_is_valid() {
    let nil_uuid = "00000000-0000-0000-0000-000000000000";
    assert!(validate_api_key(nil_uuid));
}

#[test]
fn empty_string_fails_validation() {
    assert!(!validate_api_key(""));
}

// =============================================================================
// get_valid_api_key tests
// =============================================================================

#[tokio::test]
async fn get_valid_api_key_prioritizes_headers_over_query() {
    let headers = {
        let mut map = axum::http::HeaderMap::new();
        map.insert(
            "Authorization",
            "Bearer 123e4567-e89b-12d3-a456-426614174000"
                .parse()
                .unwrap(),
        );
        map
    };
    let uri: axum::http::Uri = "/path?api_key=00000000-0000-0000-0000-000000000000"
        .parse()
        .unwrap();
    let api_key = get_valid_api_key(&headers, &uri).await.unwrap();
    assert_eq!(api_key, "123e4567-e89b-12d3-a456-426614174000".to_string());
}

#[tokio::test]
async fn invalid_header_key_falls_back_to_valid_query_key() {
    let headers = {
        let mut map = axum::http::HeaderMap::new();
        map.insert("Authorization", "Bearer invalid-key".parse().unwrap());
        map
    };
    let uri: axum::http::Uri = "/path?api_key=123e4567-e89b-12d3-a456-426614174000"
        .parse()
        .unwrap();
    let api_key = get_valid_api_key(&headers, &uri).await;
    assert!(api_key.is_none());
}

#[tokio::test]
async fn missing_header_falls_back_to_valid_query_key() {
    let headers = axum::http::HeaderMap::new();
    let uri: axum::http::Uri = "/path?api_key=123e4567-e89b-12d3-a456-426614174000"
        .parse()
        .unwrap();
    let api_key = get_valid_api_key(&headers, &uri).await.unwrap();
    assert_eq!(api_key, "123e4567-e89b-12d3-a456-426614174000");
}

#[tokio::test]
async fn both_invalid_returns_none() {
    let headers = {
        let mut map = axum::http::HeaderMap::new();
        map.insert("Authorization", "Bearer bad-key".parse().unwrap());
        map
    };
    let uri: axum::http::Uri = "/path?api_key=also-bad".parse().unwrap();
    let api_key = get_valid_api_key(&headers, &uri).await;
    assert!(api_key.is_none());
}

#[tokio::test]
async fn empty_headers_with_valid_query() {
    let headers = axum::http::HeaderMap::new();
    let uri: axum::http::Uri = "/path?api_key=550e8400-e29b-41d4-a716-446655440000"
        .parse()
        .unwrap();
    let api_key = get_valid_api_key(&headers, &uri).await.unwrap();
    assert_eq!(api_key, "550e8400-e29b-41d4-a716-446655440000");
}

#[tokio::test]
async fn no_auth_at_all_returns_none() {
    let headers = axum::http::HeaderMap::new();
    let uri: axum::http::Uri = "/path".parse().unwrap();
    let api_key = get_valid_api_key(&headers, &uri).await;
    assert!(api_key.is_none());
}
