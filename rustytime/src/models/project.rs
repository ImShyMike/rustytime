use crate::models::heartbeat::TIMEOUT_SECONDS;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::sql_types::{BigInt, Int4, Nullable, Text, Timestamptz};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::schema::projects;

use std::time::SystemTime;

struct CachedProjectId {
    id: i32,
    cached_at: SystemTime,
}

const TTL_SECONDS: u64 = 600; // 10 minutes

static PROJECT_CACHE: Lazy<Mutex<HashMap<(i32, String), CachedProjectId>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

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
    pub repo_url: Option<String>,
    pub created_at: Option<chrono::DateTime<chrono::Utc>>,
    pub updated_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(QueryableByName)]
struct ProjectWithTimeRow {
    #[diesel(sql_type = Int4)]
    id: i32,
    #[diesel(sql_type = Int4)]
    user_id: i32,
    #[diesel(sql_type = Text)]
    name: String,
    #[diesel(sql_type = Nullable<Text>)]
    repo_url: Option<String>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    created_at: Option<chrono::DateTime<chrono::Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    updated_at: Option<chrono::DateTime<chrono::Utc>>,
    #[diesel(sql_type = BigInt)]
    total_seconds: i64,
}

pub fn get_or_create_project_id(
    conn: &mut PgConnection,
    user_id_param: i32,
    project_name: &str,
    repo_url_param: Option<&str>,
) -> QueryResult<i32> {
    let now = SystemTime::now();
    {
        let mut cache = PROJECT_CACHE.lock().unwrap();
        cache.retain(|_, v| {
            v.cached_at
                .elapsed()
                .map(|e| e.as_secs() < TTL_SECONDS)
                .unwrap_or(false)
        });
        if let Some(cached) = cache.get(&(user_id_param, project_name.to_string()))
            && cached
                .cached_at
                .elapsed()
                .map(|e| e.as_secs() < TTL_SECONDS)
                .unwrap_or(false)
        {
            return Ok(cached.id);
        }
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

    PROJECT_CACHE.lock().unwrap().insert(
        (user_id_param, project_name.to_string()),
        CachedProjectId {
            id: inserted_id,
            cached_at: now,
        },
    );

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

    pub fn set_repo_url(
        conn: &mut PgConnection,
        project_id_param: i32,
        user_id_param: i32,
        new_repo_url: &Option<String>,
    ) -> QueryResult<()> {
        use crate::schema::projects::dsl::*;

        diesel::update(
            projects
                .filter(id.eq(project_id_param))
                .filter(user_id.eq(user_id_param)),
        )
        .set(repo_url.eq(new_repo_url))
        .execute(conn)?;

        Ok(())
    }

    pub fn list_projects_by_user_with_time(
        conn: &mut PgConnection,
        user_id_param: i32,
    ) -> QueryResult<Vec<(Project, i64)>> {
        let sql = r#"
            SELECT
                p.id,
                p.user_id,
                p.name,
                p.repo_url,
                p.created_at,
                p.updated_at,
                COALESCE((
                    SELECT SUM(diff)
                    FROM (
                        SELECT
                            CASE
                                WHEN prev_time IS NULL THEN 0
                                ELSE LEAST(EXTRACT(EPOCH FROM (time - prev_time)), $2)
                            END AS diff
                        FROM (
                            SELECT
                                time,
                                LAG(time) OVER (ORDER BY time) AS prev_time
                            FROM heartbeats h
                            WHERE h.user_id = $1
                              AND h.project_id IS NOT NULL
                              AND (
                                  h.project_id = p.id
                                  OR h.project_id IN (
                                      SELECT pa.project_id
                                      FROM project_aliases pa
                                      WHERE pa.user_id = $1
                                        AND pa.alias_to = p.id
                                  )
                              )
                        ) time_diffs
                    ) capped_diffs
                ), 0)::bigint AS total_seconds
            FROM projects p
            WHERE p.user_id = $1
              AND p.id NOT IN (
                  SELECT project_id
                  FROM project_aliases
                  WHERE user_id = $1
              )
            ORDER BY total_seconds DESC, p.name ASC
        "#;

        let rows = diesel::sql_query(sql)
            .bind::<Int4, _>(user_id_param)
            .bind::<Int4, _>(TIMEOUT_SECONDS)
            .load::<ProjectWithTimeRow>(conn)?;

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
                    },
                    row.total_seconds,
                )
            })
            .collect())
    }
}
