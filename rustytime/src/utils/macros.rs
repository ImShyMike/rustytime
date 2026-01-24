/// Macro to run a database query with custom error handling
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

/// Macro to run a database transaction with custom error handling
#[macro_export]
macro_rules! db_transaction {
    ($conn:expr, $body:expr) => {{
        $conn
            .build_transaction()
            .run($body)
            .map_err(|e: $crate::utils::transaction::TxError| e.into_response())?
    }};
}
