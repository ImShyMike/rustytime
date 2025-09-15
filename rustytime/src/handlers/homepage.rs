use crate::models::user::User;
use crate::state::AppState;
use axum::{Extension, Json, extract::State, response::Response};
use serde::Serialize;

#[derive(Serialize)]
pub struct HomePageResponse {
    is_authenticated: bool,
    user: Option<User>,
}

/// Handler for the homepage
pub async fn home_page(
    State(_app_state): State<AppState>,
    user: Option<Extension<User>>,
) -> Result<Json<HomePageResponse>, Response> {
    // check if user is authenticated
    let is_authenticated = user.is_some();
    let user = user.map(|ext| ext.0);

    Ok(Json(HomePageResponse {
        is_authenticated,
        user,
    }))
}
