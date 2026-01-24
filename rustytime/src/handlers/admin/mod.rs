use aide::NoApi;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

use crate::db_transaction;
use crate::models::user::User;
use crate::tx_bail;
use crate::utils::extractors::{AuthenticatedUser, DbConnection};
use crate::utils::transaction::{TxOptionExt, TxResultExt};

pub async fn change_user_admin_level(
    Path((user_id, new_level)): Path<(i32, i16)>,
    NoApi(AuthenticatedUser(current_user)): NoApi<AuthenticatedUser>,
    NoApi(DbConnection(mut conn)): NoApi<DbConnection>,
) -> Result<StatusCode, Response> {
    if !current_user.is_owner() {
        return Err((StatusCode::FORBIDDEN, "No permission").into_response());
    }

    db_transaction!(conn, |conn| {
        let target_user = User::get_by_id(conn, user_id)
            .db_err("Failed to fetch target user")?
            .or_not_found("User not found")?;

        if target_user.id == current_user.id {
            tx_bail!(StatusCode::BAD_REQUEST, "Cannot change own admin level");
        }

        if target_user.is_owner() {
            tx_bail!(StatusCode::BAD_REQUEST, "Cannot change owner admin level");
        }

        if target_user.admin_level >= current_user.admin_level {
            tx_bail!(
                StatusCode::BAD_REQUEST,
                "Cannot change admin level of equal or higher admin"
            );
        }

        User::set_admin_level(conn, user_id, new_level).db_err("Failed to update admin level")?;

        Ok(())
    });

    Ok(StatusCode::OK)
}
