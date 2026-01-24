use chrono::{DateTime, Utc};
use diesel::dsl::now;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::user::User;
use crate::schema::sessions;
use crate::schema::sessions::dsl;

pub const SESSION_EXPIRY_DAYS: i64 = 7;

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = sessions)]
pub struct Session {
    pub id: Uuid,
    pub user_id: i32,
    pub github_user_id: i64,
    pub github_access_token: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub impersonated_by: Option<i32>,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = sessions)]
pub struct NewSession {
    pub user_id: i32,
    pub github_access_token: String,
    pub github_user_id: i64,
    pub impersonated_by: Option<i32>,
}

impl Session {
    #[allow(dead_code)]
    pub fn find_by_user_id(conn: &mut PgConnection, user_id: i32) -> QueryResult<Option<Session>> {
        dsl::sessions
            .filter(dsl::user_id.eq(user_id))
            .filter(dsl::expires_at.gt(now))
            .filter(dsl::impersonated_by.is_null())
            .first::<Session>(conn)
            .optional()
    }

    pub fn find_by_github_user_id(
        conn: &mut PgConnection,
        gh_user_id: i64,
    ) -> QueryResult<Option<Session>> {
        dsl::sessions
            .filter(dsl::github_user_id.eq(gh_user_id))
            .filter(dsl::expires_at.gt(now))
            .filter(dsl::impersonated_by.is_null())
            .first::<Session>(conn)
            .optional()
    }

    pub fn create(conn: &mut PgConnection, new_session: &NewSession) -> QueryResult<Session> {
        diesel::insert_into(sessions::table)
            .values(new_session)
            .get_result(conn)
    }

    pub fn create_or_update(
        conn: &mut PgConnection,
        user_id: i32,
        access_token: &str,
        gh_user_id: i64,
    ) -> QueryResult<Session> {
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            // check if a session already exists for this user
            if let Some(existing_session) = Self::find_by_github_user_id(conn, gh_user_id)? {
                // update the existing session with new access token
                diesel::update(sessions::table.find(existing_session.id))
                    .set((
                        dsl::github_access_token.eq(access_token),
                        dsl::expires_at
                            .eq(Utc::now() + chrono::Duration::days(SESSION_EXPIRY_DAYS)),
                    ))
                    .get_result(conn)
            } else {
                // create new session
                let new_session = NewSession {
                    user_id,
                    github_access_token: access_token.to_string(),
                    github_user_id: gh_user_id,
                    impersonated_by: None,
                };
                Self::create(conn, &new_session)
            }
        })
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn delete_by_user_id(conn: &mut PgConnection, user_id: i32) -> QueryResult<usize> {
        diesel::delete(sessions::table.filter(dsl::user_id.eq(user_id))).execute(conn)
    }

    #[inline(always)]
    pub fn delete_expired(conn: &mut PgConnection) -> QueryResult<usize> {
        diesel::delete(sessions::table.filter(dsl::expires_at.le(now))).execute(conn)
    }

    pub fn impersonate(
        conn: &mut PgConnection,
        session_id: Uuid,
        target_user: &User,
        acting_admin_id: i32,
    ) -> QueryResult<Session> {
        diesel::update(sessions::table.find(session_id))
            .set((
                dsl::user_id.eq(target_user.id),
                dsl::github_user_id.eq(target_user.github_id),
                dsl::impersonated_by.eq(Some(acting_admin_id)),
            ))
            .get_result(conn)
    }

    pub fn clear_impersonation(
        conn: &mut PgConnection,
        session_id: Uuid,
        admin_user: &User,
    ) -> QueryResult<Session> {
        diesel::update(sessions::table.find(session_id))
            .set((
                dsl::user_id.eq(admin_user.id),
                dsl::github_user_id.eq(admin_user.github_id),
                dsl::impersonated_by.eq::<Option<i32>>(None),
            ))
            .get_result(conn)
    }
}
