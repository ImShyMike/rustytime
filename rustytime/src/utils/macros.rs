#[macro_export]
macro_rules! db_query {
    ($expr:expr) => {
        $expr.map_err(|e| {
            #[cfg(debug_assertions)] eprintln!("Database error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Database error").into_response()
        })?
    };
    ($expr:expr, $msg:expr) => {
        $expr.map_err(|e| {
            #[cfg(debug_assertions)] eprintln!("Database error: {:?}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, $msg).into_response()
        })?
    };
}

#[macro_export]
macro_rules! get_db_conn {
    ($state:expr) => {
        $state
            .db_pool
            .get()
            .map_err(|e| {
                #[cfg(debug_assertions)] eprintln!("Failed to connect to the database: {:?}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to connect to the database").into_response()
            })?
    };
}