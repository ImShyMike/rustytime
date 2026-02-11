use crate::db::connection::DbPool;
use crate::models::session::Session;
use crate::models::user::User;
use crate::utils::env::is_production_env;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tower_cookies::cookie::SameSite;
use tower_cookies::{Cookie, Cookies};
use uuid::Uuid;

use crate::schema::{sessions, users};

pub const SESSION_COOKIE_NAME: &str = "rustytime_session";
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
    /// Create a new session cookie
    #[inline(always)]
    pub fn create_session_cookie(session_id: Uuid, expires_at: DateTime<Utc>) -> Cookie<'static> {
        // check if in production
        let is_production = is_production_env();

        let mut cookie_builder = Cookie::build((SESSION_COOKIE_NAME, session_id.to_string()))
            .path("/")
            .expires(time::OffsetDateTime::from_unix_timestamp(expires_at.timestamp()).unwrap())
            .http_only(true)
            .same_site(SameSite::Lax);

        // in production, set secure and domain
        if is_production {
            let domain =
                std::env::var("COOKIE_DOMAIN").unwrap_or_else(|_| ".shymike.dev".to_string());
            cookie_builder = cookie_builder
                .domain(domain)
                .secure(true)
                .same_site(tower_cookies::cookie::SameSite::Lax);
        } else {
            // in development, don't set secure for localhost
            cookie_builder = cookie_builder.secure(false);
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

    #[inline(always)]
    pub fn delete_session(conn: &mut PgConnection, session_id: Uuid) -> QueryResult<usize> {
        diesel::delete(sessions::table.filter(sessions::id.eq(session_id))).execute(conn)
    }

    /// Remove the session cookie
    #[inline(always)]
    pub fn remove_session_cookie() -> Cookie<'static> {
        let is_production = is_production_env();

        let mut cookie_builder = Cookie::build((SESSION_COOKIE_NAME, ""))
            .path("/")
            .expires(time::OffsetDateTime::UNIX_EPOCH)
            .http_only(true);

        // in production, allow sharing between subdomains
        if is_production {
            let domain =
                std::env::var("COOKIE_DOMAIN").unwrap_or_else(|_| ".shymike.dev".to_string());
            cookie_builder = cookie_builder
                .domain(domain)
                .secure(true)
                .same_site(tower_cookies::cookie::SameSite::Lax);
        } else {
            cookie_builder = cookie_builder.secure(false);
        }

        cookie_builder.build()
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

#[cfg(test)]
mod tests;
