use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use uuid::Uuid;

use crate::dto::ApiResponse;

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
        let (http_status, status_code, message, message_error) = match &self {
            AppError::DatabaseError(e) => {
                tracing::error!("Database error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "99",
                    "Internal Server Error",
                    "Internal database error occurred".to_string(),
                )
            }
            AppError::InternalServerError(e) => {
                tracing::error!("Internal server error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "99",
                    "Internal Server Error",
                    e.clone(),
                )
            }
            AppError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "01",
                "Authentication Failed",
                self.to_string(),
            ),
            AppError::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                "02",
                "Unauthorized",
                self.to_string(),
            ),
            AppError::TokenExpired => (
                StatusCode::UNAUTHORIZED,
                "02",
                "Unauthorized",
                self.to_string(),
            ),
            AppError::Unauthorized(msg) => (
                StatusCode::UNAUTHORIZED,
                "02",
                "Unauthorized",
                msg.clone(),
            ),
            AppError::ValidationError(msg) => (
                StatusCode::BAD_REQUEST,
                "03",
                "Validation Error",
                msg.clone(),
            ),
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                "04",
                "Resource Not Found",
                msg.clone(),
            ),
            AppError::Forbidden(msg) => (
                StatusCode::FORBIDDEN,
                "05",
                "Access Forbidden",
                msg.clone(),
            ),
            AppError::UserAlreadyExists(msg) | AppError::Conflict(msg) => (
                StatusCode::CONFLICT,
                "09",
                "Resource Conflict",
                msg.clone(),
            ),
        };

        let reff_code = Uuid::new_v4().to_string();
        let body = ApiResponse::<()>::error(
            status_code,
            reff_code.clone(),
            message,
            message_error,
        );

        (
            http_status,
            [(
                axum::http::header::HeaderName::from_static("x-request-id"),
                reff_code,
            )],
            Json(body),
        )
            .into_response()
    }
}
