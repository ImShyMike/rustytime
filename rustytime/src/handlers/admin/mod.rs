use aide::NoApi;
use axum::extract::Path;
use axum::{
    Extension,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::models::user::User;
use crate::state::AppState;
use crate::{db_query, get_db_conn};

pub async fn change_user_admin_level(
    State(app_state): State<AppState>,
    Path((user_id, new_level)): Path<(i32, i16)>,
    user: NoApi<Option<Extension<User>>>,
) -> Result<StatusCode, Response> {
    let current_user = user
        .0
        .expect("User should be authenticated since middleware validated authentication")
        .0;

    if !current_user.is_owner() {
        return Err((StatusCode::FORBIDDEN, "No permission").into_response());
    }

    let mut conn = get_db_conn!(app_state);

    let Some(target_user) = db_query!(
        User::get_by_id(&mut conn, user_id),
        "Failed to fetch target user"
    ) else {
        return Err((StatusCode::NOT_FOUND, "User not found").into_response());
    };

    if target_user.id == current_user.id {
        return Err((StatusCode::BAD_REQUEST, "Cannot change own admin level").into_response());
    }

    if target_user.is_owner() {
        return Err((StatusCode::BAD_REQUEST, "Cannot change owner admin level").into_response());
    }

    if target_user.admin_level >= current_user.admin_level {
        return Err((
            StatusCode::BAD_REQUEST,
            "Cannot change admin level of equal or higher admin",
        )
            .into_response());
    }

    db_query!(
        User::set_admin_level(&mut conn, user_id, new_level),
        "Failed to update admin level"
    );

    Ok(StatusCode::OK)
}
