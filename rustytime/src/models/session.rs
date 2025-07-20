use chrono::{DateTime, Utc};
use diesel::dsl::now;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::sessions;
use crate::schema::sessions::dsl;

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = sessions)]
pub struct Session {
    pub id: Uuid,
    pub user_id: i32,
    pub github_access_token: String,
    pub github_user_id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = sessions)]
pub struct NewSession {
    pub user_id: i32,
    pub github_access_token: String,
    pub github_user_id: i64,
}

impl Session {
    #[allow(dead_code)]
    pub fn find_by_user_id(conn: &mut PgConnection, user_id: i32) -> QueryResult<Option<Session>> {
        dsl::sessions
            .filter(dsl::user_id.eq(user_id))
            .filter(dsl::expires_at.gt(now))
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
        // check if a session already exists for this user
        if let Some(existing_session) = Self::find_by_github_user_id(conn, gh_user_id)? {
            // update the existing session with new access token
            diesel::update(sessions::table.find(existing_session.id))
                .set((
                    dsl::github_access_token.eq(access_token),
                    dsl::updated_at.eq(now),
                ))
                .get_result(conn)
        } else {
            // create new session
            let new_session = NewSession {
                user_id,
                github_access_token: access_token.to_string(),
                github_user_id: gh_user_id,
            };
            Self::create(conn, &new_session)
        }
    }

    #[allow(dead_code)]
    pub fn delete_by_user_id(conn: &mut PgConnection, user_id: i32) -> QueryResult<usize> {
        diesel::delete(sessions::table.filter(dsl::user_id.eq(user_id))).execute(conn)
    }

    #[allow(dead_code)]
    pub fn is_expired(&self) -> bool {
        self.expires_at < Utc::now()
    }
}
