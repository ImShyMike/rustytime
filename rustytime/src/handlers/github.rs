use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{Html, Redirect},
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
    pub html_url: String,
}

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
            .unwrap_or_else(|_| "http://localhost:3000/api/v1/auth/github/callback".to_string()),
    )
    .expect("Invalid redirect URL");

    BasicClient::new(client_id)
        .set_client_secret(client_secret)
        .set_auth_uri(auth_url)
        .set_token_uri(token_url)
        .set_redirect_uri(redirect_url)
}

pub async fn login(State(app_state): State<AppState>) -> Redirect {
    let (auth_url, _csrf_token) = app_state
        .github_client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read:user".to_string()))
        .url();

    Redirect::to(auth_url.as_ref())
}

pub async fn callback(
    State(app_state): State<AppState>,
    cookies: Cookies,
    Query(params): Query<AuthRequest>,
) -> Result<Html<String>, StatusCode> {
    let code = AuthorizationCode::new(params.code);

    let http_client = reqwest::Client::new();

    // exchange code for access token
    let token_response = app_state
        .github_client
        .exchange_code(code)
        .request_async(&http_client)
        .await
        .map_err(|err| {
            eprintln!("Token exchange error: {}", err);
            StatusCode::UNAUTHORIZED
        })?;

    let access_token = token_response.access_token().secret();

    // fetch user info from GitHub
    let user_info = fetch_github_user(access_token).await.map_err(|err| {
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
        &user_info.id.to_string(),
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

    Ok(Html(format!(
        r#"
        <html>
            <head>
                <title>Welcome to rustytime</title>
                <style>
                    body {{ font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; }}
                    .user-info {{ background: #f5f5f5; padding: 20px; border-radius: 8px; margin: 20px 0; }}
                    .avatar {{ border-radius: 50%; }}
                    .api-key {{ background: #e8f4f8; padding: 10px; border-radius: 4px; font-family: monospace; }}
                    .success {{ color: #28a745; }}
                    .logout {{ margin-top: 20px; }}
                </style>
            </head>
            <body>
                <h1 class="success">✅ Welcome to rustytime, {}!</h1>
                
                <div class="user-info">
                    <img src="{}" alt="Avatar" width="100" height="100" class="avatar">
                    <h3>User Information</h3>
                    <p><strong>GitHub Username:</strong> {}</p>
                    <p><strong>GitHub ID:</strong> {}</p>
                    <p><strong>Profile:</strong> <a href="{}" target="_blank">{}</a></p>
                    <p><strong>Account Created:</strong> {}</p>
                </div>

                <div class="api-key">
                    <h3>Your API Key</h3>
                    <p><strong>API Key:</strong> {}</p>
                    <p><small>Use this key to authenticate with the API</small></p>
                </div>
            </body>
        </html>
        "#,
        user_info.login,
        user_info.avatar_url,
        user_info.login,
        user_info.id,
        user_info.html_url,
        user_info.html_url,
        user.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
        user.api_key
    )))
}

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

pub async fn dashboard(
    State(app_state): State<AppState>,
    cookies: Cookies,
) -> Result<Html<String>, StatusCode> {
    // get current user from session
    let user = SessionManager::get_current_user(&cookies, &app_state.db_pool)
        .await
        .map_err(|err| {
            eprintln!("Database error: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // get user's session info
    let session_id =
        SessionManager::get_session_from_cookies(&cookies).ok_or(StatusCode::UNAUTHORIZED)?;

    let session_data = SessionManager::validate_session(&app_state.db_pool, session_id)
        .await
        .map_err(|err| {
            eprintln!("Session validation error: {}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    Ok(Html(format!(
        r#"
        <html>
            <head>
                <title>rustytime Dashboard</title>
                <style>
                    body {{ font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; }}
                    .user-info {{ background: #f5f5f5; padding: 20px; border-radius: 8px; margin: 20px 0; }}
                    .avatar {{ border-radius: 50%; }}
                    .api-key {{ background: #e8f4f8; padding: 10px; border-radius: 4px; font-family: monospace; }}
                    .success {{ color: #28a745; }}
                    .logout {{ margin-top: 20px; }}
                    .logout a {{ background: #dc3545; color: white; padding: 10px 20px; text-decoration: none; border-radius: 4px; }}
                </style>
            </head>
            <body>
                <h1 class="success">🎯 rustytime Dashboard</h1>
                
                <div class="user-info">
                    <img src="{}" alt="Avatar" width="100" height="100" class="avatar">
                    <h3>User Information</h3>
                    <p><strong>Name:</strong> {}</p>
                    <p><strong>GitHub ID:</strong> {}</p>
                    <p><strong>Account Created:</strong> {}</p>
                    <p><strong>Session Expires:</strong> {}</p>
                </div>

                <div class="api-key">
                    <h3>Your API Key</h3>
                    <p><strong>API Key:</strong> {}</p>
                    <p><small>Use this key to authenticate with the API</small></p>
                </div>

                <div class="logout">
                    <a href="/api/v1/auth/github/logout">Logout</a>
                </div>
            </body>
        </html>
        "#,
        user.avatar_url.as_deref().unwrap_or(""),
        user.name.as_deref().unwrap_or("Unknown"),
        session_data.github_user_id,
        user.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
        session_data.expires_at.format("%Y-%m-%d %H:%M:%S UTC"),
        user.api_key
    )))
}

async fn fetch_github_user(token: &str) -> Result<GitHubUser, reqwest::Error> {
    let client = reqwest::Client::new();
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
