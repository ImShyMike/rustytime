#![forbid(unsafe_code)]
#![cfg(feature = "integration")]

pub mod db;
pub mod docs;
pub mod handlers;
pub mod jobs;
pub mod models;
pub mod routes;
pub mod schema;
pub mod state;
pub mod utils;

use std::sync::LazyLock;
use std::time::Instant;
pub static START_TIME: LazyLock<Instant> = LazyLock::new(Instant::now);
