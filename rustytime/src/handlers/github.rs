use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointNotSet, EndpointSet,
    RedirectUrl, Scope, TokenResponse, TokenUrl, basic::BasicClient, reqwest,
};
use serde::{Deserialize, Serialize};
use std::env;
use tower_cookies::Cookies;

use crate::state::AppState;
use crate::utils::session::SessionManager;
use crate::{get_db_conn, models::session::Session};
use crate::{models::user::User, utils::env::is_production_env};
use axum::Json;

#[derive(Deserialize)]
pub struct AuthRequest {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct GitHubUser {
    pub login: String,
    pub id: u64,
    pub avatar_url: String,
}

#[derive(Serialize)]
pub struct AuthUrlResponse {
    pub auth_url: String,
}

#[derive(Serialize)]
pub struct CallbackUserResponse {
    pub id: i32,
    pub github_id: u64,
    pub name: String,
    pub avatar_url: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub github_id: i64,
    pub username: String,
    pub avatar_url: String,
    pub admin_level: i16,
}

#[derive(Serialize)]
pub struct ImpersonationResponse {
    pub admin_id: i32,
    pub admin_name: String,
    pub admin_avatar_url: String,
}

#[derive(Serialize)]
pub struct VerifySessionResponse {
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<UserResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub impersonation: Option<ImpersonationResponse>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
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
pub async fn login(State(app_state): State<AppState>, cookies: Cookies) -> Json<AuthUrlResponse> {
    let csrf_token = CsrfToken::new_random();
    let csrf_token_secret = csrf_token.secret().clone();
    let (auth_url, _) = app_state
        .github_client
        .authorize_url(|| csrf_token)
        .add_scope(Scope::new("read:user".to_string()))
        .url();

    // check if in production for cookie security settings
    let is_production = is_production_env();

    let mut cookie = tower_cookies::Cookie::build(("rustytime_oauth_state", csrf_token_secret))
        .path("/")
        .http_only(true)
        .secure(is_production) // only secure in production
        .same_site(tower_cookies::cookie::SameSite::Lax)
        .build();

    cookie.set_expires(time::OffsetDateTime::now_utc() + time::Duration::minutes(10));
    cookies.add(cookie);

    Json(AuthUrlResponse {
        auth_url: auth_url.to_string(),
    })
}

/// Handler for GitHub OAuth callback
pub async fn callback(
    State(app_state): State<AppState>,
    cookies: Cookies,
    Query(params): Query<AuthRequest>,
) -> Result<Redirect, Response> {
    // get frontend URL from environment variable
    let frontend_url =
        env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());

    // validate state parameter against cookie
    let state_cookie = cookies.get("rustytime_oauth_state");
    let state_valid = state_cookie
        .as_ref()
        .map(|c| c.value() == params.state)
        .unwrap_or(false);

    // remove the oauth state cookie after checking
    let remove_state_cookie = tower_cookies::Cookie::build(("rustytime_oauth_state", ""))
        .path("/")
        .build();
    cookies.remove(remove_state_cookie);

    if !state_valid {
        return Ok(Redirect::to(&format!(
            "{}/?error=invalid_state",
            frontend_url
        )));
    }

    let code = AuthorizationCode::new(params.code.clone());

    // exchange code for access token
    let token_response = match app_state
        .github_client
        .exchange_code(code)
        .request_async(&app_state.http_client)
        .await
    {
        Ok(response) => response,
        Err(_) => {
            return Ok(Redirect::to(&format!(
                "{}/?error=token_exchange",
                frontend_url
            )));
        }
    };

    let access_token = token_response.access_token().secret();

    // fetch user info from GitHub
    let user_info = match fetch_github_user(&app_state.http_client, access_token).await {
        Ok(info) => info,
        Err(_) => return Ok(Redirect::to(&format!("{}/?error=github_api", frontend_url))),
    };

    // get database connection
    let mut conn = get_db_conn!(app_state);

    let (user, session) = match conn.transaction::<_, diesel::result::Error, _>(|conn| {
        // create or update user in database
        let user = User::create_or_update(
            conn,
            user_info.id as i64,
            &user_info.login,
            &user_info.avatar_url,
        )?;

        // create or update session for authentication
        let session = Session::create_or_update(conn, user.id, access_token, user_info.id as i64)?;

        Ok((user, session))
    }) {
        Ok(result) => result,
        Err(_) => return Ok(Redirect::to(&format!("{}/?error=database", frontend_url))),
    };

    // set session cookie
    let session_cookie = SessionManager::create_session_cookie(session.id);
    cookies.add(session_cookie);

    // create JSON response
    let user_data = CallbackUserResponse {
        id: user.id,
        github_id: user_info.id,
        name: user_info.login,
        avatar_url: user_info.avatar_url,
    };

    let user_string = serde_json::to_string(&user_data).unwrap_or_default();
    let user_encoded = urlencoding::encode(&user_string);

    Ok(Redirect::to(&format!(
        "{}/?session_id={}&user={}",
        frontend_url, session.id, user_encoded
    )))
}

/// Handler to verify a session token
pub async fn verify_session(
    State(app_state): State<AppState>,
    Query(params): Query<serde_json::Value>,
    cookies: Cookies,
) -> Result<Json<VerifySessionResponse>, Response> {
    let session_id = params
        .get("session_id")
        .and_then(|v| v.as_str())
        .and_then(|s| uuid::Uuid::parse_str(s).ok())
        .or_else(|| {
            cookies
                .get("rustytime_session")
                .and_then(|cookie| uuid::Uuid::parse_str(cookie.value()).ok())
        })
        .ok_or_else(|| {
            (StatusCode::BAD_REQUEST, "Missing or invalid session_id").into_response()
        })?;

    match SessionManager::validate_session(&app_state.db_pool, session_id).await {
        Ok(Some(session_data)) => {
            // get user details
            let mut conn = get_db_conn!(app_state);
            let user = crate::schema::users::table
                .find(session_data.user_id)
                .first::<User>(&mut conn)
                .map_err(|err| {
                    eprintln!("Database error fetching user: {}", err);
                    (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
                })?;

            let impersonation = if let Some(admin_id) = session_data.impersonated_by {
                match crate::schema::users::table
                    .find(admin_id)
                    .first::<User>(&mut conn)
                    .optional()
                {
                    Ok(Some(admin)) => Some(ImpersonationResponse {
                        admin_id: admin.id,
                        admin_name: admin.name,
                        admin_avatar_url: admin.avatar_url,
                    }),
                    Ok(None) => None,
                    Err(err) => {
                        eprintln!("Database error fetching impersonating admin: {}", err);
                        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
                            .into_response());
                    }
                }
            } else {
                None
            };

            Ok(Json(VerifySessionResponse {
                valid: true,
                user: Some(UserResponse {
                    id: user.id,
                    github_id: user.github_id,
                    username: user.name,
                    avatar_url: user.avatar_url,
                    admin_level: user.admin_level,
                }),
                impersonation,
                expires_at: Some(session_data.expires_at),
                message: None,
            }))
        }
        Ok(None) => Ok(Json(VerifySessionResponse {
            valid: false,
            user: None,
            impersonation: None,
            expires_at: None,
            message: Some("Session not found or expired".to_string()),
        })),
        Err(err) => {
            eprintln!("Database error validating session: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response())
        }
    }
}

/// Handler to log out the user
pub async fn logout(
    State(app_state): State<AppState>,
    cookies: Cookies,
) -> Result<Response, Response> {
    // get session from cookie
    if let Some(session_id) = SessionManager::get_session_from_cookies(&cookies) {
        // delete session from database
        let mut conn = get_db_conn!(app_state);

        diesel::delete(
            crate::schema::sessions::table.filter(crate::schema::sessions::id.eq(session_id)),
        )
        .execute(&mut conn)
        .map_err(|err| {
            eprintln!("Database error deleting session: {}", err);
            (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error").into_response()
        })?;
    }

    // remove session cookie
    let remove_cookie = SessionManager::remove_session_cookie();
    cookies.add(remove_cookie);

    Ok(StatusCode::OK.into_response())
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
