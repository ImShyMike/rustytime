use axum::Extension;
use axum::extract::FromRequestParts;
use axum::http::StatusCode;
use axum::http::request::Parts;

use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::PooledConnection;

use crate::models::user::User;
use crate::state::AppState;

/// Custom extractor for authenticated users
pub struct AuthenticatedUser(pub User);

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        Extension::<User>::from_request_parts(parts, state)
            .await
            .map(|Extension(user)| AuthenticatedUser(user))
            .map_err(|e| {
                tracing::error!("❌ Failed to extract authenticated user: {:?}", e);
                StatusCode::UNAUTHORIZED
            })
    }
}

/// Extractor for database connections
pub struct DbConnection(pub PooledConnection<ConnectionManager<PgConnection>>);

impl FromRequestParts<AppState> for DbConnection {
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(
        _parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        state.db_pool.get().map(DbConnection).map_err(|e| {
            tracing::error!("❌ Failed to get database connection: {:?}", e);
            (
                StatusCode::SERVICE_UNAVAILABLE,
                "Failed to get database connection",
            )
        })
    }
}
