use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::users;

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub github_id: i64,
    pub name: String,
    pub avatar_url: String,
    pub api_key: Uuid,
    pub is_admin: bool,
    pub is_banned: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Insertable, Deserialize, Debug)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub github_id: i64,
    pub name: String,
    pub avatar_url: String,
    pub is_admin: bool,
    pub is_banned: bool,
}

impl User {
    pub fn find_by_github_id(conn: &mut PgConnection, github_id: i64) -> QueryResult<Option<User>> {
        users::table
            .filter(users::github_id.eq(github_id))
            .first::<User>(conn)
            .optional()
    }

    pub fn get_by_id(conn: &mut PgConnection, user_id: i32) -> QueryResult<Option<User>> {
        users::table.find(user_id).first::<User>(conn).optional()
    }

    pub fn create(conn: &mut PgConnection, new_user: &NewUser) -> QueryResult<User> {
        diesel::insert_into(users::table)
            .values(new_user)
            .get_result(conn)
    }

    pub fn create_or_update(
        conn: &mut PgConnection,
        github_id: i64,
        username: &str,
        avatar_url: &str,
    ) -> QueryResult<User> {
        // first, try to find existing user by github_id
        if let Some(existing_user) = Self::find_by_github_id(conn, github_id)? {
            // update user info if it has changed
            if existing_user.avatar_url != avatar_url || existing_user.name != username {
                diesel::update(users::table.find(existing_user.id))
                    .set((users::avatar_url.eq(avatar_url), users::name.eq(username)))
                    .get_result(conn)
            } else {
                Ok(existing_user)
            }
        } else {
            let total_users = Self::count_total_users(conn, true)?;

            // create new user
            let new_user = NewUser {
                github_id,
                name: username.to_string(),
                avatar_url: avatar_url.to_string(),
                is_admin: total_users == 0, // make the first real user an admin
                is_banned: false,
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

    #[allow(dead_code)]
    pub fn list_admins(conn: &mut PgConnection) -> QueryResult<Vec<User>> {
        users::table
            .filter(users::is_admin.eq(true))
            .load::<User>(conn)
    }

    pub fn count_total_users(conn: &mut PgConnection, only_real: bool) -> QueryResult<i64> {
        if only_real {
            users::table
                .count()
                .filter(users::github_id.gt(0))
                .get_result(conn)
        } else {
            users::table.count().get_result(conn)
        }
    }
}
