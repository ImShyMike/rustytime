use crate::schema::project_aliases;
use chrono::{DateTime, Utc};
use diesel::prelude::*;

#[derive(Queryable)]
#[allow(dead_code)]
pub struct ProjectAlias {
    pub id: i32,
    pub user_id: i32,
    pub project_id: i32,
    pub alias_to: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = project_aliases)]
pub struct NewProjectAlias {
    pub user_id: i32,
    pub project_id: i32,
    pub alias_to: i32,
}

impl ProjectAlias {
    pub fn list_user_project_aliases(
        conn: &mut PgConnection,
        user_id_param: i32,
    ) -> QueryResult<Vec<ProjectAlias>> {
        project_aliases::table
            .filter(project_aliases::user_id.eq(user_id_param))
            .load::<ProjectAlias>(conn)
    }

    pub fn create(
        conn: &mut PgConnection,
        new_alias: &NewProjectAlias,
    ) -> QueryResult<ProjectAlias> {
        diesel::insert_into(project_aliases::table)
            .values(new_alias)
            .get_result(conn)
    }

    pub fn delete_project_alias(
        conn: &mut PgConnection,
        user_id_param: i32,
        alias_id_param: i32,
    ) -> QueryResult<usize> {
        diesel::delete(
            project_aliases::table
                .filter(project_aliases::user_id.eq(user_id_param))
                .filter(project_aliases::project_id.eq(alias_id_param)),
        )
        .execute(conn)
    }
}
