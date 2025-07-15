use axum::extract::Path;

pub async fn list_projects() -> String {
    "projects".to_string()
}

pub async fn get_project(Path(id): Path<String>) -> String {
    format!("project: {}", id)
}
