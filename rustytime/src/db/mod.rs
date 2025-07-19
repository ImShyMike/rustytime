use diesel::pg::PgConnection;
use diesel::r2d2::{self, ConnectionManager};
use dotenvy::dotenv;
use std::env;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn create_pool() -> DbPool {
    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").unwrap_or_else(|_| "postgres://localhost/rustytime".to_string());

    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}
