use axum::extract::Path;

pub async fn list_users() -> String {
    "users".to_string()
}

pub async fn get_user(Path(id): Path<String>) -> String {
    format!("user: {}", id)
}
