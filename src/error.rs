use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Build a standard application error that can be resolved by error_catalog.
/// Without that middleware, its body still renders the per-code constant default.
pub fn catalog_error_response(
    http_status: StatusCode,
    status_code: &'static str,
    message: &'static str,
) -> Response {
    crate::middleware::error_catalog::response(http_status, status_code, message)
}

#[derive(Debug)]
pub enum AppError {
    DatabaseError(sqlx::Error),
    InternalServerError(String),
    InvalidCredentials,
    UserAlreadyExists(String),
    NotFound(String),
    InvalidToken,
    TokenExpired,
    Unauthorized(String),
    Forbidden(String),
    ValidationError(String),
    Conflict(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::DatabaseError(e) => write!(f, "Database error: {}", e),
            AppError::InternalServerError(e) => write!(f, "Internal server error: {}", e),
            AppError::InvalidCredentials => write!(f, "Invalid username/email or password"),
            AppError::UserAlreadyExists(e) => write!(f, "User already exists: {}", e),
            AppError::NotFound(e) => write!(f, "Resource not found: {}", e),
            AppError::InvalidToken => write!(f, "Invalid authentication token"),
            AppError::TokenExpired => write!(f, "Authentication token has expired"),
            AppError::Unauthorized(e) => write!(f, "Unauthorized: {}", e),
            AppError::Forbidden(e) => write!(f, "Forbidden: {}", e),
            AppError::ValidationError(e) => write!(f, "Validation error: {}", e),
            AppError::Conflict(e) => write!(f, "Conflict: {}", e),
        }
    }
}

impl std::error::Error for AppError {}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        if let sqlx::Error::RowNotFound = err {
            return AppError::NotFound("Resource not found".to_string());
        }
        if let Some(db_err) = err.as_database_error() {
            if db_err.is_unique_violation() {
                return AppError::Conflict(db_err.message().to_string());
            }
            if db_err.is_foreign_key_violation() {
                return AppError::ValidationError(format!(
                    "Referenced resource invalid: {}",
                    db_err.message()
                ));
            }
        }
        AppError::DatabaseError(err)
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        use jsonwebtoken::errors::ErrorKind;
        match err.kind() {
            ErrorKind::ExpiredSignature => AppError::TokenExpired,
            _ => AppError::InvalidToken,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (http_status, status_code, message) = match &self {
            AppError::DatabaseError(error) => {
                tracing::error!("Database error: {:?}", error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "99",
                    "Internal Server Error",
                )
            }
            AppError::InternalServerError(error) => {
                tracing::error!("Internal server error: {}", error);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "99",
                    "Internal Server Error",
                )
            }
            AppError::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "01", "Authentication Failed")
            }
            AppError::InvalidToken | AppError::TokenExpired | AppError::Unauthorized(_) => {
                (StatusCode::UNAUTHORIZED, "02", "Unauthorized")
            }
            AppError::ValidationError(_) => (StatusCode::BAD_REQUEST, "03", "Validation Error"),
            AppError::NotFound(_) => (StatusCode::NOT_FOUND, "04", "Resource Not Found"),
            AppError::Forbidden(_) => (StatusCode::FORBIDDEN, "05", "Access Forbidden"),
            AppError::UserAlreadyExists(_) | AppError::Conflict(_) => {
                (StatusCode::CONFLICT, "09", "Resource Conflict")
            }
        };
        catalog_error_response(http_status, status_code, message)
    }
}
