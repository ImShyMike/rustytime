use crate::models::heartbeat::TIMEOUT_SECONDS;
use diesel::insert_into;
use diesel::prelude::*;
use diesel::sql_types::Int4;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::schema::projects;

use std::time::SystemTime;

diesel::define_sql_function! {
    /// Calculate project time with alias resolution
    fn calculate_project_time_with_aliases(
        user_id: Int4,
        project_id: Int4,
        timeout_seconds: Int4
    ) -> diesel::sql_types::BigInt;
}

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
        use crate::schema::projects::dsl::*;

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            // get projects that are not aliases
            let project_list: Vec<Project> = projects
                .filter(user_id.eq(user_id_param))
                .filter(
                    id.ne_all(
                        crate::schema::project_aliases::table
                            .filter(crate::schema::project_aliases::user_id.eq(user_id_param))
                            .select(crate::schema::project_aliases::project_id),
                    ),
                )
                .load::<Project>(conn)?;

            // calculate time for each project
            let mut results = Vec::with_capacity(project_list.len());
            for project in project_list {
                let total_seconds: i64 = diesel::select(calculate_project_time_with_aliases(
                    user_id_param,
                    project.id,
                    TIMEOUT_SECONDS,
                ))
                .get_result(conn)?;

                results.push((project, total_seconds));
            }

            // sort by total_seconds DESC, then by name ASC
            results.sort_by(
                |(a_proj, a_time), (b_proj, b_time)| match b_time.cmp(a_time) {
                    std::cmp::Ordering::Equal => a_proj.name.cmp(&b_proj.name),
                    other => other,
                },
            );

            Ok(results)
        })
    }
}
