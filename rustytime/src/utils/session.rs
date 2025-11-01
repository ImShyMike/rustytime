use crate::db::connection::DbPool;
use crate::models::session::Session;
use crate::models::user::User;
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tower_cookies::cookie::SameSite;
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;

use crate::schema::{sessions, users};

pub const SESSION_COOKIE_NAME: &str = "rustytime_session";
pub const SESSION_DURATION_DAYS: i64 = 30;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionData {
    pub id: Uuid,
    pub user_id: i32,
    pub github_user_id: i64,
    pub expires_at: DateTime<Utc>,
    pub impersonated_by: Option<i32>,
}

pub struct SessionManager;

#[derive(Debug, Clone)]
pub struct ResolvedSession {
    #[allow(dead_code)]
    pub session: SessionData,
    pub user: User,
    pub impersonator: Option<User>,
}

#[derive(Debug, Clone)]
pub struct ImpersonationContext {
    pub admin: User,
}

impl SessionManager {
    #[inline(always)]
    pub(crate) fn is_production_env() -> bool {
        if let Ok(env) = std::env::var("ENVIRONMENT") {
            if env.eq_ignore_ascii_case("production") {
                return true;
            }
        }

        if let Ok(prod) = std::env::var("PRODUCTION") {
            let normalized = prod.trim().to_ascii_lowercase();
            return matches!(normalized.as_str(), "true" | "1" | "yes");
        }

        false
    }

    /// Create a new session cookie
    #[inline(always)]
    pub fn create_session_cookie(session_id: Uuid) -> Cookie<'static> {
        let expires = Utc::now() + Duration::days(SESSION_DURATION_DAYS);

        // check if in production
        let is_production = Self::is_production_env();

        let mut cookie_builder = Cookie::build((SESSION_COOKIE_NAME, session_id.to_string()))
            .path("/")
            .expires(time::OffsetDateTime::from_unix_timestamp(expires.timestamp()).unwrap())
            .http_only(true);

        // in production, use SameSite::None for cross-origin requests (Cloudflare Workers)
        if is_production {
            let domain =
                std::env::var("COOKIE_DOMAIN").unwrap_or_else(|_| ".shymike.dev".to_string());
            cookie_builder = cookie_builder
                .domain(domain)
                .secure(true)
                .same_site(SameSite::None);
        } else {
            // in development, use Lax for localhost
            cookie_builder = cookie_builder.secure(false).same_site(SameSite::Lax);
        }

        cookie_builder.build()
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
            id: s.id,
            user_id: s.user_id,
            github_user_id: s.github_user_id,
            expires_at: s.expires_at,
            impersonated_by: s.impersonated_by,
        }))
    }

    /// Remove the session cookie
    #[inline(always)]
    pub fn remove_session_cookie() -> Cookie<'static> {
        let is_production = Self::is_production_env();

        let mut cookie_builder = Cookie::build((SESSION_COOKIE_NAME, ""))
            .path("/")
            .expires(time::OffsetDateTime::UNIX_EPOCH)
            .http_only(true);

        // in production, match the same settings as create_session_cookie
        if is_production {
            let domain =
                std::env::var("COOKIE_DOMAIN").unwrap_or_else(|_| ".shymike.dev".to_string());
            cookie_builder = cookie_builder
                .domain(domain)
                .secure(true)
                .same_site(SameSite::None);
        } else {
            cookie_builder = cookie_builder.secure(false).same_site(SameSite::Lax);
        }

        cookie_builder.build()
    }

    /// Check if user is authenticated
    #[allow(dead_code)]
    #[inline(always)]
    pub async fn is_authenticated(cookies: &Cookies, pool: &DbPool) -> bool {
        matches!(Self::resolve_session(cookies, pool).await, Ok(Some(_)))
    }

    /// Try to get the current user using the session cookie
    #[allow(dead_code)]
    #[inline(always)]
    pub async fn get_current_user(
        cookies: &Cookies,
        pool: &DbPool,
    ) -> Result<Option<User>, diesel::result::Error> {
        Ok(Self::resolve_session(cookies, pool)
            .await?
            .map(|resolved| resolved.user))
    }

    pub async fn resolve_session(
        cookies: &Cookies,
        pool: &DbPool,
    ) -> Result<Option<ResolvedSession>, diesel::result::Error> {
        let Some(session_id) = Self::get_session_from_cookies(cookies) else {
            return Ok(None);
        };

        let Some(session_data) = Self::validate_session(pool, session_id).await? else {
            return Ok(None);
        };

        let mut conn = pool
            .get()
            .map_err(|_| diesel::result::Error::BrokenTransactionManager)?;

        let user = users::table
            .find(session_data.user_id)
            .first::<User>(&mut conn)
            .optional()?;

        let Some(user) = user else {
            return Ok(None);
        };

        let impersonator = if let Some(admin_id) = session_data.impersonated_by {
            users::table
                .find(admin_id)
                .first::<User>(&mut conn)
                .optional()?
        } else {
            None
        };

        Ok(Some(ResolvedSession {
            session: session_data,
            user,
            impersonator,
        }))
    }
}
