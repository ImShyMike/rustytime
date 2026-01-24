use chrono::{DateTime, Utc};
use diesel::prelude::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::schema::users::{self};

#[derive(Queryable, Selectable, Serialize, Deserialize, Debug, Clone)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub github_id: i64,
    pub name: String,
    pub avatar_url: String,
    pub api_key: Uuid,
    pub admin_level: i16,
    pub is_banned: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub timezone: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct PartialUser {
    pub id: i32,
    pub github_id: i64,
    pub name: String,
    pub avatar_url: String,
    #[schemars(with = "Option<String>")]
    pub api_key: Option<Uuid>,
    pub admin_level: i16,
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
    pub admin_level: i16,
    pub is_banned: bool,
    pub timezone: String,
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
        conn.transaction::<_, diesel::result::Error, _>(|conn| {
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
                    admin_level: if total_users == 0 { 2 } else { 0 }, // make the first real user an owner
                    is_banned: false,
                    timezone: "UTC".to_string(),
                };
                Self::create(conn, &new_user)
            }
        })
    }

    pub fn is_admin(&self) -> bool {
        self.admin_level > 0
    }

    pub fn is_owner(&self) -> bool {
        self.admin_level > 1
    }

    pub fn list_users_paginated(
        conn: &mut PgConnection,
        limit: i64,
        offset: i64,
    ) -> QueryResult<Vec<User>> {
        users::table
            .order(users::admin_level.desc())
            .then_order_by(users::id.asc())
            .limit(limit)
            .offset(offset)
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

    pub fn set_admin_level(
        conn: &mut PgConnection,
        user_id: i32,
        new_level: i16,
    ) -> QueryResult<usize> {
        diesel::update(users::table.find(user_id))
            .set(users::admin_level.eq(new_level))
            .execute(conn)
    }

    pub fn set_timezone(
        conn: &mut PgConnection,
        user_id: i32,
        timezone: &str,
    ) -> QueryResult<User> {
        diesel::update(users::table.find(user_id))
            .set(users::timezone.eq(timezone))
            .get_result(conn)
    }
}
