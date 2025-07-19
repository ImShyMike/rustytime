use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::users;

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub api_key: Uuid,
    pub github_id: i32,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub github_id: i32,
}

impl User {
    pub fn find_by_github_id(conn: &mut PgConnection, github_id: i32) -> QueryResult<Option<User>> {
        users::table
            .filter(users::github_id.eq(github_id))
            .first::<User>(conn)
            .optional()
    }

    pub fn create(conn: &mut PgConnection, new_user: &NewUser) -> QueryResult<User> {
        diesel::insert_into(users::table)
            .values(new_user)
            .get_result(conn)
    }

    pub fn create_or_update(
        conn: &mut PgConnection,
        github_id: i32,
        username: &str,
        avatar_url: &str,
    ) -> QueryResult<User> {
        // first, try to find existing user by github_id
        if let Some(existing_user) = Self::find_by_github_id(conn, github_id)? {
            // update user info if it has changed
            if existing_user.avatar_url.as_deref() != Some(avatar_url)
                || existing_user.name.as_deref() != Some(username)
            {
                diesel::update(users::table.find(existing_user.id))
                    .set((users::avatar_url.eq(avatar_url), users::name.eq(username)))
                    .get_result(conn)
            } else {
                Ok(existing_user)
            }
        } else {
            // create new user
            let new_user = NewUser {
                github_id,
                name: Some(username.to_string()),
                avatar_url: Some(avatar_url.to_string()),
            };
            Self::create(conn, &new_user)
        }
    }
}
