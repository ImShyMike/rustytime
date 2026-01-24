#[macro_export]
macro_rules! db_query {
    ($expr:expr) => {
        $expr.map_err(|_e| {
            tracing::error!("❌ Database error: {:?}", _e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An unknown database error has occured",
            )
                .into_response()
        })?
    };
    ($expr:expr, $msg:expr) => {
        $expr.map_err(|_e| {
            tracing::error!("❌ Database error: {:?}", _e);
            (StatusCode::INTERNAL_SERVER_ERROR, $msg).into_response()
        })?
    };
}
