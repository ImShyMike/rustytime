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
    pub is_admin: bool,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub github_id: i32,
    pub is_admin: bool,
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
            let total_users = Self::count_total_users(conn)?;

            // create new user
            let new_user = NewUser {
                github_id,
                name: Some(username.to_string()),
                avatar_url: Some(avatar_url.to_string()),
                is_admin: total_users == 0, // make the first user an admin
            };
            Self::create(conn, &new_user)
        }
    }

    pub fn is_admin(&self) -> bool {
        self.is_admin
    }

    pub fn list_all_users(conn: &mut PgConnection) -> QueryResult<Vec<User>> {
        users::table.load::<User>(conn)
    }

    pub fn list_admins(conn: &mut PgConnection) -> QueryResult<Vec<User>> {
        users::table
            .filter(users::is_admin.eq(true))
            .load::<User>(conn)
    }

    pub fn count_total_users(conn: &mut PgConnection) -> QueryResult<i64> {
        users::table.count().get_result(conn)
    }
}
