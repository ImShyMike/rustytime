use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::Redirect,
};
use diesel::prelude::*;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet,
    RedirectUrl, Scope, TokenResponse, TokenUrl, basic::BasicClient, reqwest,
};
use serde::Deserialize;
use std::env;
use tower_cookies::Cookies;

use crate::models::session::Session;
use crate::models::user::User;
use crate::state::AppState;
use crate::utils::session::SessionManager;

#[derive(Deserialize)]
pub struct AuthRequest {
    pub code: String,
    #[allow(dead_code)]
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    pub avatar_url: String,
    // pub html_url: String,
}

/// Create a new GitHub OAuth client
pub fn create_github_client()
-> BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet> {
    let client_id = ClientId::new(env::var("GITHUB_CLIENT_ID").expect("Missing GITHUB_CLIENT_ID"));
    let client_secret =
        ClientSecret::new(env::var("GITHUB_CLIENT_SECRET").expect("Missing GITHUB_CLIENT_SECRET"));
    let auth_url = AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
        .expect("Invalid token endpoint URL");

    let redirect_url = RedirectUrl::new(
        env::var("REDIRECT_URL")
            .unwrap_or_else(|_| "http://localhost:3000/auth/github/callback".to_string()),
    )
    .expect("Invalid redirect URL");

    BasicClient::new(client_id)
        .set_client_secret(client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(redirect_url)
}

/// Handler to initiate GitHub OAuth login
pub async fn login(State(app_state): State<AppState>) -> Redirect {
    let (auth_url, _csrf_token) = app_state
        .github_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read:user".to_string()))
        .url();

    Redirect::to(auth_url.as_ref())
}

/// Handler for GitHub OAuth callback
pub async fn callback(
    State(app_state): State<AppState>,
    cookies: Cookies,
    Query(params): Query<AuthRequest>,
) -> Result<Redirect, StatusCode> {
    let code = AuthorizationCode::new(params.code);

    // exchange code for access token
    let token_response = app_state
        .github_client
        .exchange_code(code)
        .request_async(&app_state.http_client)
        .await
        .map_err(|err| {
            eprintln!("Token exchange error: {}", err);
            StatusCode::UNAUTHORIZED
        })?;

    let access_token = token_response.access_token().secret();

    // fetch user info from GitHub
    let user_info = fetch_github_user(&app_state.http_client, access_token)
        .await
        .map_err(|err| {
            eprintln!("GitHub API error: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // get database connection
    let mut conn = app_state.db_pool.get().map_err(|err| {
        eprintln!("Database connection error: {}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // create or update user in database
    let user = User::create_or_update(
        &mut conn,
        user_info.id as i32,
        &user_info.login,
        &user_info.avatar_url,
    )
    .map_err(|err| {
        eprintln!("Database error creating/updating user: {}", err);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // create or update session for authentication
    let session = Session::create_or_update(&mut conn, user.id, access_token, user_info.id as i64)
        .map_err(|err| {
            eprintln!("Database error creating/updating session: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // set session cookie
    let session_cookie = SessionManager::create_session_cookie(session.id);
    cookies.add(session_cookie);

    Ok(axum::response::Redirect::to("/dashboard"))
}

/// Handler to log out the user
pub async fn logout(
    State(app_state): State<AppState>,
    cookies: Cookies,
) -> Result<Redirect, StatusCode> {
    // get session from cookie
    if let Some(session_id) = SessionManager::get_session_from_cookies(&cookies) {
        // delete session from database
        let mut conn = app_state.db_pool.get().map_err(|err| {
            eprintln!("Database connection error: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

        diesel::delete(
            crate::schema::sessions::table.filter(crate::schema::sessions::id.eq(session_id)),
        )
        .execute(&mut conn)
        .map_err(|err| {
            eprintln!("Database error deleting session: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    }

    // remove session cookie
    let remove_cookie = SessionManager::remove_session_cookie();
    cookies.add(remove_cookie);

    Ok(Redirect::to("/"))
}

/// Fetch GitHub user information using the access token
async fn fetch_github_user(
    client: &reqwest::Client,
    token: &str,
) -> Result<GitHubUser, reqwest::Error> {
    let user: GitHubUser = client
        .get("https://api.github.com/user")
        .bearer_auth(token)
        .header("User-Agent", "rustytime-github-oauth")
        .send()
        .await?
        .json()
        .await?;
    Ok(user)
}
