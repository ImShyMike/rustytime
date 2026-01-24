use axum::response::Redirect;

/// Handler for the homepage
pub async fn home_page() -> Result<Redirect, Redirect> {
    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());
    Ok(Redirect::to(&frontend_url))
}
