use crate::state::AppState;
use axum::{extract::State, response::Redirect};
/// Handler for the homepage
pub async fn home_page(State(_app_state): State<AppState>) -> Result<Redirect, Redirect> {
    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    Ok(Redirect::to(&frontend_url))
}
