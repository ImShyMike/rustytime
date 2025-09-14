use crate::db::DbPool;
use crate::models::session::Session;
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tower_cookies::cookie::SameSite;
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;

use crate::models::user::User;
use crate::schema::{sessions, users};

pub const SESSION_COOKIE_NAME: &str = "rustytime_session";
pub const SESSION_DURATION_DAYS: i64 = 30;

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionData {
    pub user_id: i32,
    pub github_user_id: i64,
    pub expires_at: DateTime<Utc>,
}

pub struct SessionManager;

impl SessionManager {
    /// Create a new session cookie
    #[inline(always)]
    pub fn create_session_cookie(session_id: Uuid) -> Cookie<'static> {
        let expires = Utc::now() + Duration::days(SESSION_DURATION_DAYS);

        Cookie::build((SESSION_COOKIE_NAME, session_id.to_string()))
            .path("/")
            .expires(time::OffsetDateTime::from_unix_timestamp(expires.timestamp()).unwrap())
            .http_only(true)
            .secure(true)
            .same_site(SameSite::Lax)
            .build()
    }

    /// Get session from cookie
    #[inline(always)]
    pub fn get_session_from_cookies(cookies: &Cookies) -> Option<Uuid> {
        cookies
            .get(SESSION_COOKIE_NAME)
            .and_then(|cookie| Uuid::parse_str(cookie.value()).ok())
    }

    /// Validate session and return user info
    pub async fn validate_session(
        pool: &DbPool,
        session_id: Uuid,
    ) -> Result<Option<SessionData>, diesel::result::Error> {
        let mut conn = pool
            .get()
            .map_err(|_| diesel::result::Error::BrokenTransactionManager)?;

        let session = sessions::table
            .find(session_id)
            .filter(sessions::expires_at.gt(diesel::dsl::now))
            .first::<Session>(&mut conn)
            .optional()?;

        Ok(session.map(|s| SessionData {
            user_id: s.user_id,
            github_user_id: s.github_user_id,
            expires_at: s.expires_at,
        }))
    }

    /// Remove the session cookie
    #[inline(always)]
    pub fn remove_session_cookie() -> Cookie<'static> {
        Cookie::build((SESSION_COOKIE_NAME, ""))
            .path("/")
            .expires(time::OffsetDateTime::UNIX_EPOCH)
            .http_only(true)
            .build()
    }

    /// Check if user is authenticated
    #[allow(dead_code)]
    #[inline(always)]
    pub async fn is_authenticated(cookies: &Cookies, pool: &DbPool) -> bool {
        if let Some(session_id) = Self::get_session_from_cookies(cookies) {
            if let Ok(Some(_)) = Self::validate_session(pool, session_id).await {
                return true;
            }
        }
        false
    }

    /// Try to get the current user using the session cookie
    pub async fn get_current_user(
        cookies: &Cookies,
        pool: &DbPool,
    ) -> Result<Option<User>, diesel::result::Error> {
        if let Some(session_id) = Self::get_session_from_cookies(cookies) {
            if let Some(session_data) = Self::validate_session(pool, session_id).await? {
                let mut conn = pool
                    .get()
                    .map_err(|_| diesel::result::Error::BrokenTransactionManager)?;

                let user = users::table
                    .find(session_data.user_id)
                    .first::<User>(&mut conn)
                    .optional()?;

                return Ok(user);
            }
        }
        Ok(None)
    }
}
