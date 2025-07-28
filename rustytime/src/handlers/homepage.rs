use axum::{
    Extension,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use minijinja::context;

use crate::models::user::User;
use crate::state::AppState;

/// Handler for the homepage (will likely be done using SvelteKit later)
pub async fn home_page(
    State(app_state): State<AppState>,
    user: Option<Extension<User>>,
) -> Result<Html<String>, Response> {
    // check if user is authenticated
    let is_authenticated = user.is_some();

    let is_admin = user.as_ref().is_some_and(|u| u.is_admin());

    let rendered = app_state
        .template_engine
        .render(
            "home.html",
            context! {
                is_authenticated => is_authenticated,
                is_admin => is_admin
            },
        )
        .map_err(|err| {
            eprintln!("Template rendering error: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
        })?;

    Ok(Html(rendered))
}
