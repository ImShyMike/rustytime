use chrono::{DateTime, Utc};
use diesel::dsl::now;
use diesel::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

use crate::models::user::User;
use crate::schema::sessions;
use crate::schema::sessions::dsl;
use crate::utils::instrumented;

#[derive(Queryable, Selectable, Debug, Clone)]
#[diesel(table_name = sessions)]
#[allow(dead_code)]
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
        instrumented::first("Session::find_by_user_id", || {
            dsl::sessions
                .filter(dsl::user_id.eq(user_id))
                .filter(dsl::expires_at.gt(now))
                .filter(dsl::impersonated_by.is_null())
                .first::<Session>(conn)
        })
        .optional()
    }

    pub fn create(conn: &mut PgConnection, new_session: &NewSession) -> QueryResult<Session> {
        instrumented::first("Session::create", || {
            diesel::insert_into(sessions::table)
                .values(new_session)
                .get_result(conn)
        })
    }

    #[allow(dead_code)]
    #[inline(always)]
    pub fn delete_by_user_id(conn: &mut PgConnection, user_id: i32) -> QueryResult<usize> {
        instrumented::execute("Session::delete_by_user_id", || {
            diesel::delete(sessions::table.filter(dsl::user_id.eq(user_id))).execute(conn)
        })
    }

    pub fn scrub_expired(conn: &mut PgConnection) -> QueryResult<usize> {
        instrumented::execute("Session::scrub_expired", || {
            diesel::update(
                sessions::table
                    .filter(dsl::expires_at.lt(diesel::dsl::now))
                    .filter(dsl::github_access_token.ne("")),
            )
            .set(dsl::github_access_token.eq(""))
            .execute(conn)
        })
    }

    pub fn delete_stale(conn: &mut PgConnection, retention_days: i64) -> QueryResult<usize> {
        instrumented::execute("Session::delete_stale", || {
            diesel::delete(
                sessions::table.filter(
                    dsl::expires_at.lt(Utc::now() - chrono::Duration::days(retention_days)),
                ),
            )
            .execute(conn)
        })
    }

    pub fn impersonate(
        conn: &mut PgConnection,
        session_id: Uuid,
        target_user: &User,
        acting_admin_id: i32,
    ) -> QueryResult<Session> {
        instrumented::first("Session::impersonate", || {
            diesel::update(
                sessions::table
                    .filter(dsl::id.eq(session_id))
                    .filter(dsl::user_id.eq(acting_admin_id))
                    .filter(dsl::impersonated_by.is_null()),
            )
            .set((
                dsl::user_id.eq(target_user.id),
                dsl::github_user_id.eq(target_user.github_id),
                dsl::impersonated_by.eq(Some(acting_admin_id)),
            ))
            .get_result(conn)
        })
    }

    pub fn clear_impersonation(
        conn: &mut PgConnection,
        session_id: Uuid,
        admin_user: &User,
    ) -> QueryResult<Session> {
        instrumented::first("Session::clear_impersonation", || {
            diesel::update(
                sessions::table
                    .filter(dsl::id.eq(session_id))
                    .filter(dsl::impersonated_by.eq(admin_user.id)),
            )
            .set((
                dsl::user_id.eq(admin_user.id),
                dsl::github_user_id.eq(admin_user.github_id),
                dsl::impersonated_by.eq::<Option<i32>>(None),
            ))
            .get_result(conn)
        })
    }
}
