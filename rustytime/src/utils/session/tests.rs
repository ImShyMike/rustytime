use super::*;

#[test]
fn create_session_cookie_sets_secure_attributes_in_production() {
    let session_id = Uuid::new_v4();
    let expires_at = Utc::now() + chrono::Duration::days(7);
    let cookie = SessionManager::create_session_cookie(session_id, expires_at);
    if is_production_env() {
        assert!(cookie.secure().unwrap_or(false));
        assert_eq!(cookie.same_site().unwrap_or(SameSite::Lax), SameSite::Lax);
    } else {
        assert!(!cookie.secure().unwrap_or(true));
    }
}

#[test]
fn create_session_cookie_has_correct_name() {
    let session_id = Uuid::new_v4();
    let expires_at = Utc::now() + chrono::Duration::days(7);
    let cookie = SessionManager::create_session_cookie(session_id, expires_at);
    assert_eq!(cookie.name(), SESSION_COOKIE_NAME);
}

#[test]
fn create_session_cookie_has_correct_path() {
    let session_id = Uuid::new_v4();
    let expires_at = Utc::now() + chrono::Duration::days(7);
    let cookie = SessionManager::create_session_cookie(session_id, expires_at);
    assert_eq!(cookie.path(), Some("/"));
}

#[test]
fn create_session_cookie_is_http_only() {
    let session_id = Uuid::new_v4();
    let expires_at = Utc::now() + chrono::Duration::days(7);
    let cookie = SessionManager::create_session_cookie(session_id, expires_at);
    assert!(cookie.http_only().unwrap_or(false));
}

#[test]
fn create_session_cookie_value_matches_session_id() {
    let session_id = Uuid::new_v4();
    let expires_at = Utc::now() + chrono::Duration::days(7);
    let cookie = SessionManager::create_session_cookie(session_id, expires_at);
    assert_eq!(cookie.value(), session_id.to_string());
}

#[test]
fn create_session_cookie_same_site_is_lax() {
    let session_id = Uuid::new_v4();
    let expires_at = Utc::now() + chrono::Duration::days(7);
    let cookie = SessionManager::create_session_cookie(session_id, expires_at);
    assert_eq!(cookie.same_site(), Some(SameSite::Lax));
}

#[test]
fn get_session_from_cookies_parses_uuid() {
    let session_id = Uuid::new_v4();
    let cookies = Cookies::default();
    cookies.add(Cookie::new(SESSION_COOKIE_NAME, session_id.to_string()));
    let extracted = SessionManager::get_session_from_cookies(&cookies);
    assert_eq!(extracted, Some(session_id));
}

#[test]
fn get_session_from_cookies_missing_cookie_returns_none() {
    let cookies = Cookies::default();
    let extracted = SessionManager::get_session_from_cookies(&cookies);
    assert_eq!(extracted, None);
}

#[test]
fn get_session_from_cookies_invalid_uuid_returns_none() {
    let cookies = Cookies::default();
    cookies.add(Cookie::new(SESSION_COOKIE_NAME, "not-a-valid-uuid"));
    let extracted = SessionManager::get_session_from_cookies(&cookies);
    assert_eq!(extracted, None);
}

#[test]
fn get_session_from_cookies_empty_value_returns_none() {
    let cookies = Cookies::default();
    cookies.add(Cookie::new(SESSION_COOKIE_NAME, ""));
    let extracted = SessionManager::get_session_from_cookies(&cookies);
    assert_eq!(extracted, None);
}

#[test]
fn remove_session_cookie_has_correct_name() {
    let cookie = SessionManager::remove_session_cookie();
    assert_eq!(cookie.name(), SESSION_COOKIE_NAME);
}

#[test]
fn remove_session_cookie_has_empty_value() {
    let cookie = SessionManager::remove_session_cookie();
    assert_eq!(cookie.value(), "");
}

#[test]
fn remove_session_cookie_expires_at_unix_epoch() {
    let cookie = SessionManager::remove_session_cookie();
    assert_eq!(
        cookie.expires_datetime(),
        Some(time::OffsetDateTime::UNIX_EPOCH)
    );
}

#[test]
fn remove_session_cookie_has_correct_path() {
    let cookie = SessionManager::remove_session_cookie();
    assert_eq!(cookie.path(), Some("/"));
}
