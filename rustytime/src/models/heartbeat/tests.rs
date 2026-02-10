use super::*;
use serde_json::json;

// ============================================================================
// f64_to_datetime tests
// ============================================================================

#[test]
fn f64_to_datetime_zero_timestamp() {
    let dt = f64_to_datetime(0.0);
    assert_eq!(dt.timestamp(), 0);
    assert_eq!(dt.timestamp_subsec_nanos(), 0);
}

#[test]
fn f64_to_datetime_negative_timestamp() {
    let dt = f64_to_datetime(-86400.0);
    assert_eq!(dt.timestamp(), -86400);
}

#[test]
fn f64_to_datetime_round_trip_with_datetime_to_f64() {
    let now = Utc::now();
    let timestamp = datetime_to_f64(now);
    let back = f64_to_datetime(timestamp);
    let diff = (datetime_to_f64(back) - timestamp).abs();
    assert!(diff < 1e-6, "Round-trip difference too large: {diff}");
}

#[test]
fn f64_to_datetime_max_reasonable_timestamp() {
    let timestamp = 4102444800.0; // 2100-01-01 00:00:00 UTC
    let dt = f64_to_datetime(timestamp);
    assert_eq!(dt.timestamp(), 4102444800);
}

// ============================================================================
// HeartbeatRequest deserialization tests
// ============================================================================

#[test]
fn heartbeat_request_minimal_valid_payload() {
    let payload = json!({
        "entity": "test.rs",
        "type": "file",
        "time": 1700000000.0
    });
    let request: HeartbeatRequest = serde_json::from_value(payload).unwrap();
    assert_eq!(request.entity, "test.rs");
    assert_eq!(request.type_, "file");
    assert!((request.time - 1700000000.0).abs() < f64::EPSILON);
    assert!(request.project.is_none());
    assert!(request.language.is_none());
}

#[test]
fn heartbeat_request_all_fields_populated() {
    let payload = json!({
        "entity": "main.rs",
        "type": "file",
        "time": 1700000000.5,
        "category": "coding",
        "project": "MyProject",
        "project_root_count": 2,
        "branch": "feature-branch",
        "language": "Rust",
        "dependencies": ["serde", "tokio"],
        "lines": 500,
        "line_additions": 20,
        "line_deletions": 10,
        "lineno": 42,
        "cursorpos": 100,
        "is_write": true,
        "plugin": "wakatime-plugin",
        "user_agent": "wakatime/1.0"
    });
    let request: HeartbeatRequest = serde_json::from_value(payload).unwrap();
    assert_eq!(request.entity, "main.rs");
    assert_eq!(request.category, Some("coding".to_string()));
    assert_eq!(request.project, Some("MyProject".to_string()));
    assert_eq!(request.project_root_count, Some(2));
    assert_eq!(request.branch, Some("feature-branch".to_string()));
    assert_eq!(request.language, Some("Rust".to_string()));
    assert_eq!(
        request.dependencies,
        Some(vec!["serde".to_string(), "tokio".to_string()])
    );
    assert_eq!(request.lines, Some(500));
    assert_eq!(request.line_additions, Some(20));
    assert_eq!(request.line_deletions, Some(10));
    assert_eq!(request.lineno, Some(42));
    assert_eq!(request.cursorpos, Some(100));
    assert_eq!(request.is_write, Some(true));
    assert_eq!(request.plugin, Some("wakatime-plugin".to_string()));
    assert_eq!(request.user_agent, Some("wakatime/1.0".to_string()));
}

#[test]
fn heartbeat_request_missing_optional_fields() {
    let payload = json!({
        "entity": "file.py",
        "type": "file",
        "time": 1700000000.0,
        "project": null,
        "language": null
    });
    let request: HeartbeatRequest = serde_json::from_value(payload).unwrap();
    assert_eq!(request.entity, "file.py");
    assert!(request.project.is_none());
    assert!(request.language.is_none());
    assert!(request.branch.is_none());
    assert!(request.dependencies.is_none());
}

// ============================================================================
// SanitizedHeartbeatRequest tests
// ============================================================================

#[test]
fn sanitized_heartbeat_request_category_defaults_to_coding() {
    let request = HeartbeatRequest {
        entity: "test.rs".to_string(),
        type_: "file".to_string(),
        time: 1700000000.0,
        category: None,
        project: None,
        project_root_count: None,
        branch: None,
        language: None,
        dependencies: None,
        lines: None,
        line_additions: None,
        line_deletions: None,
        lineno: None,
        cursorpos: None,
        is_write: None,
        plugin: None,
        user_agent: None,
    };
    let sanitized = SanitizedHeartbeatRequest::from_request(request);
    assert_eq!(sanitized.category, Some("coding".to_string()));
}

#[test]
fn sanitized_heartbeat_request_all_truncations_applied() {
    let long_entity = "x".repeat(MAX_ENTITY_LENGTH + 100);
    let long_type = "y".repeat(MAX_TYPE_LENGTH + 100);
    let long_project = "z".repeat(MAX_PROJECT_LENGTH + 100);
    let long_branch = "b".repeat(MAX_BRANCH_LENGTH + 100);
    let long_language = "l".repeat(MAX_LANGUAGE_LENGTH + 100);
    let long_category = "c".repeat(MAX_CATEGORY_LENGTH + 100);

    let request = HeartbeatRequest {
        entity: long_entity,
        type_: long_type,
        time: 1700000000.0,
        category: Some(long_category),
        project: Some(long_project),
        project_root_count: Some(1),
        branch: Some(long_branch),
        language: Some(long_language),
        dependencies: None,
        lines: None,
        line_additions: None,
        line_deletions: None,
        lineno: None,
        cursorpos: None,
        is_write: None,
        plugin: None,
        user_agent: None,
    };

    let sanitized = SanitizedHeartbeatRequest::from_request(request);

    assert_eq!(sanitized.entity.chars().count(), MAX_ENTITY_LENGTH);
    assert_eq!(sanitized.type_.chars().count(), MAX_TYPE_LENGTH);
    assert_eq!(
        sanitized.project.as_ref().unwrap().chars().count(),
        MAX_PROJECT_LENGTH
    );
    assert_eq!(
        sanitized.branch.as_ref().unwrap().chars().count(),
        MAX_BRANCH_LENGTH
    );
    assert_eq!(
        sanitized.language.as_ref().unwrap().chars().count(),
        MAX_LANGUAGE_LENGTH
    );
    assert_eq!(
        sanitized.category.as_ref().unwrap().chars().count(),
        MAX_CATEGORY_LENGTH
    );
}

#[test]
fn sanitized_heartbeat_request_infers_defaults() {
    let request = sample_request();
    let sanitized = SanitizedHeartbeatRequest::from_request(request);
    assert_eq!(sanitized.category.unwrap(), "coding");
}

#[test]
fn sanitized_heartbeat_into_new_heartbeat_maps_headers() {
    let request = sample_request();
    let sanitized = SanitizedHeartbeatRequest::from_request(request);
    let headers = HeaderMap::new();
    let new_heartbeat = sanitized.into_new_heartbeat(1, "1.1.1.1".parse().unwrap(), &headers);
    assert_eq!(new_heartbeat.user_id, 1);
    assert_eq!(new_heartbeat.entity, "example.txt".to_string());
    assert_eq!(new_heartbeat.type_, "file".to_string());
    assert_eq!(new_heartbeat.ip_address, "1.1.1.1".parse().unwrap());
}

// ============================================================================
// String truncation tests
// ============================================================================

fn sample_request() -> HeartbeatRequest {
    HeartbeatRequest {
        entity: "example.txt".to_string(),
        type_: "file".to_string(),
        time: 1_700_000_000.123456,
        category: None,
        project: Some("ExampleProject".to_string()),
        project_root_count: Some(1),
        branch: Some("main".to_string()),
        language: Some("Rust".to_string()),
        dependencies: Some(vec!["dep1".to_string(), "dep2".to_string()]),
        lines: Some(100),
        line_additions: Some(10),
        line_deletions: Some(5),
        lineno: Some(42),
        cursorpos: Some(128),
        is_write: Some(true),
        plugin: None,
        user_agent: None,
    }
}

#[test]
fn truncates_strings_without_breaking_utf8_boundaries() {
    let input = "😀😀😀😀😀😀😀😀😀😀".to_string();
    let truncated = truncate_string(input, 7);
    assert_eq!(truncated.chars().count(), 7);
}

#[test]
fn truncates_optional_strings_when_present() {
    let input = Some("Hello 😀, World!".to_string());
    let truncated = truncate_optional_string(input, 10);
    assert_eq!(truncated.unwrap().chars().count(), 10);
}

// ============================================================================
// datetime_to_f64 tests
// ============================================================================

#[test]
fn converts_datetime_to_f64_with_milisecond_precision() {
    let dt = DateTime::<Utc>::from_timestamp(1_700_000_000, 987_000_000).expect("valid timestamp");

    let converted = datetime_to_f64(dt);
    let expected = 1_700_000_000f64 + 987_000_000f64 / 1e9;
    let diff = (converted - expected).abs();

    assert!(
        diff <= 1e-12,
        "expected {expected}, got {converted}, diff {diff}"
    );
}

#[test]
fn datetime_to_f64_rounds_to_nearest_millisecond() {
    let dt = DateTime::<Utc>::from_timestamp(1_700_000_000, 987_354_321).expect("valid timestamp");

    let converted = datetime_to_f64(dt);
    let expected = 1_700_000_000f64 + 987_000_000f64 / 1e9;
    let diff = (converted - expected).abs();

    assert!(
        diff <= 1e-12,
        "expected {expected}, got {converted}, diff {diff}"
    );
}

// ============================================================================
// NewHeartbeat tests
// ============================================================================

#[test]
fn new_heartbeat_constructor_applies_truncation() {
    let entity = "a".repeat(MAX_ENTITY_LENGTH + 10);
    let type_ = "b".repeat(MAX_TYPE_LENGTH + 10);
    let new_heartbeat = NewHeartbeat::new(
        Utc::now(),
        1,
        entity.clone(),
        type_.clone(),
        "1.1.1.1".parse().unwrap(),
    );
    assert_eq!(new_heartbeat.entity.chars().count(), MAX_ENTITY_LENGTH);
    assert_eq!(new_heartbeat.type_.chars().count(), MAX_TYPE_LENGTH);
}

#[test]
fn new_heartbeat_from_request_round_trips_request_payload() {
    let request = sample_request();
    let headers = HeaderMap::new();
    let new_heartbeat =
        NewHeartbeat::from_request(request, 1, "1.1.1.1".parse().unwrap(), &headers);
    assert_eq!(new_heartbeat.entity, "example.txt".to_string());
    assert_eq!(new_heartbeat.type_, "file".to_string());
    assert_eq!(new_heartbeat.project.unwrap(), "ExampleProject".to_string());
    assert_eq!(new_heartbeat.language.unwrap(), "Rust".to_string());
    assert_eq!(new_heartbeat.lines.unwrap(), 100);
}

// ============================================================================
// HeartbeatResponse tests
// ============================================================================

#[test]
fn heartbeat_response_conversion_retains_entity_details() {
    let heartbeat = Heartbeat {
        id: 1,
        time: Utc::now(),
        created_at: Utc::now(),
        user_id: 1,
        entity: "test.txt".to_string(),
        type_: "file".to_string(),
        ip_address: "127.0.0.1/32".parse().unwrap(),
        project: None,
        branch: None,
        category: None,
        cursorpos: None,
        dependencies: None,
        editor: None,
        is_write: None,
        language: None,
        line_additions: None,
        line_deletions: None,
        lines: None,
        machine: None,
        operating_system: None,
        project_id: None,
        project_root_count: None,
        user_agent: "".to_string(),
        lineno: None,
        source_type: None,
    };
    let response = HeartbeatResponse::from(heartbeat.clone());
    assert_eq!(response.id, heartbeat.id);
}

// ============================================================================
// HackatimeHeartbeat tests
// ============================================================================

#[test]
fn hackatime_heartbeat_handles_missing_fields() {
    let heartbeat = HackatimeHeartbeat {
        id: 42,
        user_id: 7,
        branch: None,
        category: None,
        dependencies: None,
        editor: None,
        entity: "missing.rs".to_string(),
        language: None,
        machine: None,
        operating_system: None,
        project: None,
        type_: "file".to_string(),
        user_agent: None,
        line_additions: None,
        line_deletions: None,
        lineno: None,
        lines: None,
        cursorpos: None,
        project_root_count: None,
        time: 1_700_000_000.0,
        is_write: None,
        created_at: None,
        updated_at: None,
        fields_hash: None,
        source_type: None,
        ip_address: None,
        ysws_program: None,
        deleted_at: None,
        raw_data: serde_json::Value::Null,
        raw_heartbeat_upload_id: None,
    };

    let new_heartbeat = heartbeat.to_new_heartbeat(99);

    assert_eq!(new_heartbeat.user_agent, "");
    assert_eq!(new_heartbeat.ip_address, "127.0.0.1/32".parse().unwrap());
    assert_eq!(
        new_heartbeat.source_type,
        Some(SourceType::HackatimeImport as i16)
    );
}

fn minimal_hackatime_payload() -> serde_json::Value {
    json!({
        "id": 1,
        "user_id": 1,
        "entity": "main.rs",
        "type": "file",
        "time": 1_700_000_000.0
    })
}

#[test]
fn hackatime_time_accepts_stringified_numbers() {
    let mut payload = minimal_hackatime_payload();
    payload["time"] = json!("1700000000.5");

    let parsed: HackatimeHeartbeat = serde_json::from_value(payload).expect("time as string");
    assert!((parsed.time - 1_700_000_000.5).abs() < f64::EPSILON);
}

#[test]
fn hackatime_time_accepts_rfc3339_strings() {
    let mut payload = minimal_hackatime_payload();
    payload["time"] = json!("2024-11-28T12:34:56.789Z");

    let parsed: HackatimeHeartbeat = serde_json::from_value(payload).expect("time as RFC3339");
    assert_eq!(datetime_to_f64(f64_to_datetime(parsed.time)), parsed.time);
}
