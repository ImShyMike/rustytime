use axum::{Extension, extract::State, http::StatusCode, response::Html};
use minijinja::context;

use crate::models::user::User;
use crate::state::AppState;

/// Handler for the homepage (will likely be done using SvelteKit later)
pub async fn home_page(
    State(app_state): State<AppState>,
    user: Option<Extension<User>>,
) -> Result<Html<String>, StatusCode> {
    // check if user is authenticated
    let is_authenticated = user.is_some();

    let rendered = app_state
        .template_engine
        .render(
            "home.html",
            context! {
                is_authenticated => is_authenticated
            },
        )
        .map_err(|err| {
            eprintln!("Template rendering error: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok(Html(rendered))
}
