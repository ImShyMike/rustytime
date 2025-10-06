use diesel::insert_into;
use diesel::prelude::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

use crate::schema::projects;

static PROJECT_CACHE: Lazy<Mutex<HashMap<(i32, String), i32>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Insertable)]
#[diesel(table_name = projects)]
struct NewProject<'a> {
    user_id: i32,
    name: &'a str,
    repo_url: Option<&'a str>,
}

pub fn get_or_create_project_id(
    conn: &mut PgConnection,
    user_id_param: i32,
    project_name: &str,
    repo_url_param: Option<&str>,
) -> QueryResult<i32> {
    {
        let cache = PROJECT_CACHE.lock().unwrap();
        if let Some(&cached_id) = cache.get(&(user_id_param, project_name.to_string())) {
            return Ok(cached_id);
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
        .do_nothing()
        .returning(id)
        .get_result(conn)
        .or_else(|_| {
            projects
                .filter(user_id.eq(user_id_param))
                .filter(name.eq(project_name))
                .select(id)
                .first(conn)
        })?;

    PROJECT_CACHE
        .lock()
        .unwrap()
        .insert((user_id_param, project_name.to_string()), inserted_id);

    Ok(inserted_id)
}
