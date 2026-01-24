use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use diesel::result::Error;

/// Error type for database transactions
pub enum TxError {
    /// A database error with a custom message
    Db { error: Error, message: &'static str },
    /// An axum Response
    Response(Box<Response>),
}

impl From<Error> for TxError {
    fn from(error: Error) -> Self {
        TxError::Db {
            error,
            message: "Database error",
        }
    }
}

#[allow(dead_code)]
impl TxError {
    pub fn into_response(self) -> Response {
        match self {
            TxError::Db { error, message } => {
                tracing::error!("❌ Database error: {:?}", error);
                (StatusCode::INTERNAL_SERVER_ERROR, message).into_response()
            }
            TxError::Response(r) => *r,
        }
    }

    pub fn response(status: StatusCode, message: &'static str) -> Self {
        TxError::Response(Box::new((status, message).into_response()))
    }

    pub fn not_found(message: &'static str) -> Self {
        Self::response(StatusCode::NOT_FOUND, message)
    }

    pub fn bad_request(message: &'static str) -> Self {
        Self::response(StatusCode::BAD_REQUEST, message)
    }

    pub fn conflict(message: &'static str) -> Self {
        Self::response(StatusCode::CONFLICT, message)
    }

    pub fn conflict_owned(message: String) -> Self {
        TxError::Response(Box::new((StatusCode::CONFLICT, message).into_response()))
    }
}

/// Extension trait for Option to convert None to TxError
#[allow(dead_code)]
pub trait TxOptionExt<T> {
    fn or_not_found(self, msg: &'static str) -> Result<T, TxError>;
    fn or_bad_request(self, msg: &'static str) -> Result<T, TxError>;
    fn or_status(self, status: StatusCode, msg: &'static str) -> Result<T, TxError>;
}

impl<T> TxOptionExt<T> for Option<T> {
    fn or_not_found(self, msg: &'static str) -> Result<T, TxError> {
        self.ok_or_else(|| TxError::not_found(msg))
    }

    fn or_bad_request(self, msg: &'static str) -> Result<T, TxError> {
        self.ok_or_else(|| TxError::bad_request(msg))
    }

    fn or_status(self, status: StatusCode, msg: &'static str) -> Result<T, TxError> {
        self.ok_or_else(|| TxError::response(status, msg))
    }
}

/// Extension trait for diesel Results to add custom error messages
pub trait TxResultExt<T> {
    fn db_err(self, msg: &'static str) -> Result<T, TxError>;
}

impl<T> TxResultExt<T> for Result<T, Error> {
    fn db_err(self, msg: &'static str) -> Result<T, TxError> {
        self.map_err(|error| TxError::Db {
            error,
            message: msg,
        })
    }
}

/// Macro to exit out of a transaction with a TxError response
#[macro_export]
macro_rules! tx_bail {
    ($status:expr, $msg:expr) => {
        return Err($crate::utils::transaction::TxError::response($status, $msg))
    };
}
