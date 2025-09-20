#[macro_export]
macro_rules! db_query {
    ($expr:expr) => {
        $expr.map_err(|_e| {
            #[cfg(debug_assertions)]
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
            #[cfg(debug_assertions)]
            tracing::error!("❌ Database error: {:?}", _e);
            (StatusCode::INTERNAL_SERVER_ERROR, $msg).into_response()
        })?
    };
}

#[macro_export]
macro_rules! get_db_conn {
    ($state:expr) => {
        $state.db_pool.get().map_err(|_e| {
            #[cfg(debug_assertions)]
            tracing::error!("❌ Failed to get database connection: {:?}", _e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to get database connection",
            )
                .into_response()
        })?
    };
}
