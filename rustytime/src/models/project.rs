use std::sync::Arc;

use crate::models::heartbeat::TIMEOUT_SECONDS;
use crate::utils::cache::HeartbeatProjectCacheKey;
use diesel::QueryableByName;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Bool, Int4, Nullable as SqlNullable, Text, Timestamptz};
use moka::sync::Cache;
use once_cell::sync::Lazy;

use crate::schema::projects;

static PROJECT_CACHE: Lazy<Arc<Cache<HeartbeatProjectCacheKey, i32>>> = Lazy::new(|| {
    Arc::new(
        Cache::builder()
            .max_capacity(1_000)
            .time_to_live(std::time::Duration::from_secs(600)) // 10 minute TTL
            .build(),
    )
});

diesel::define_sql_function! {
    fn list_projects_with_time(
        user_id: Int4,
        timeout_seconds: Int4
    ) -> diesel::sql_types::Array<
        diesel::sql_types::Record<(
            Int4,
            Int4,
            Text,
            SqlNullable<Text>,
            SqlNullable<Timestamptz>,
            SqlNullable<Timestamptz>,
            BigInt,
        )>
    >;
}

#[derive(Insertable)]
#[diesel(table_name = projects)]
struct NewProject<'a> {
    user_id: i32,
    name: &'a str,
    repo_url: Option<&'a str>,
}

#[derive(Queryable, Debug, Clone)]
pub struct Project {
    pub id: i32,
    pub user_id: i32,
    pub name: String,
    #[allow(dead_code)]
    pub repo_url: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
    pub hidden: bool,
    pub project_url: Option<String>,
}

#[derive(QueryableByName)]
struct ProjectWithTimeRow {
    #[diesel(sql_type = Int4)]
    id: i32,
    #[diesel(sql_type = Int4)]
    user_id: i32,
    #[diesel(sql_type = Text)]
    name: String,
    #[diesel(sql_type = SqlNullable<Text>)]
    repo_url: Option<String>,
    #[diesel(sql_type = SqlNullable<Timestamptz>)]
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[diesel(sql_type = SqlNullable<Timestamptz>)]
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
    #[diesel(sql_type = BigInt)]
    total_seconds: i64,
    #[diesel(sql_type = Bool)]
    hidden: bool,
    #[diesel(sql_type = SqlNullable<Text>)]
    project_url: Option<String>,
}

pub fn get_or_create_project_id(
    conn: &mut PgConnection,
    user_id_param: i32,
    project_name: &str,
    repo_url_param: Option<&str>,
) -> QueryResult<i32> {
    let cache_key = HeartbeatProjectCacheKey {
        user_id: user_id_param,
        project_name: project_name.to_string(),
    };

    if let Some(cached_id) = PROJECT_CACHE.get(&cache_key) {
        return Ok(cached_id);
    }

    use crate::schema::projects::dsl::*;

    let new_project = NewProject {
        user_id: user_id_param,
        name: project_name,
        repo_url: repo_url_param,
    };

    let inserted_id: i32 = insert_into(projects)
        .values(&new_project)
        .on_conflict((user_id, name))
        .do_update()
        .set(updated_at.eq(diesel::dsl::now))
        .returning(id)
        .get_result(conn)
        .or_else(|_| {
            projects
                .filter(user_id.eq(user_id_param))
                .filter(name.eq(project_name))
                .select(id)
                .first(conn)
        })?;

    PROJECT_CACHE.insert(cache_key, inserted_id);

    Ok(inserted_id)
}

impl Project {
    pub fn list_user_projects(
        conn: &mut PgConnection,
        user_id_param: i32,
    ) -> QueryResult<Vec<Project>> {
        use crate::schema::projects::dsl::*;

        projects
            .filter(user_id.eq(user_id_param))
            .order(name.asc())
            .load::<Project>(conn)
    }

    pub fn set_project_url(
        conn: &mut PgConnection,
        project_id_param: i32,
        user_id_param: i32,
        new_project_url: &Option<String>,
    ) -> QueryResult<()> {
        use crate::schema::projects::dsl::*;

        diesel::update(
            projects
                .filter(id.eq(project_id_param))
                .filter(user_id.eq(user_id_param)),
        )
        .set(project_url.eq(new_project_url))
        .execute(conn)?;

        Ok(())
    }

    pub fn list_projects_by_user_with_time(
        conn: &mut PgConnection,
        user_id_param: i32,
    ) -> QueryResult<Vec<(Project, i64)>> {
        let rows: Vec<ProjectWithTimeRow> = diesel::sql_query(
            "SELECT id, user_id, name, repo_url, created_at, updated_at, total_seconds, hidden, project_url \
             FROM list_projects_with_time($1, $2)",
        )
        .bind::<Int4, _>(user_id_param)
        .bind::<Int4, _>(TIMEOUT_SECONDS)
        .load(conn)?;

        Ok(rows
            .into_iter()
            .map(|row| {
                (
                    Project {
                        id: row.id,
                        user_id: row.user_id,
                        name: row.name,
                        repo_url: row.repo_url,
                        created_at: row.created_at,
                        updated_at: row.updated_at,
                        hidden: row.hidden,
                        project_url: row.project_url,
                    },
                    row.total_seconds,
                )
            })
            .collect())
    }

    pub fn set_hidden(
        conn: &mut PgConnection,
        project_id_param: i32,
        user_id_param: i32,
        new_hidden: bool,
    ) -> QueryResult<()> {
        use crate::schema::projects::dsl::*;

        diesel::update(
            projects
                .filter(id.eq(project_id_param))
                .filter(user_id.eq(user_id_param)),
        )
        .set(hidden.eq(new_hidden))
        .execute(conn)?;

        Ok(())
    }
}
