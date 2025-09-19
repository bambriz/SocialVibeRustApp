use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;
use std::fmt;

// Content moderation error type with detailed violation information
#[derive(Debug, Clone)]
pub struct ContentModerationError {
    pub message: String,
    pub violation_type: Option<String>,
    pub details: Option<String>,
}

// Application-wide error type
#[derive(Debug)]
pub enum AppError {
    DatabaseError(String),
    ValidationError(String),
    ContentModerationError(ContentModerationError),
    AuthError(String),
    NotFound(String),
    InternalError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            AppError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            AppError::ContentModerationError(err) => write!(f, "Content moderation error: {}", err.message),
            AppError::AuthError(msg) => write!(f, "Authentication error: {}", msg),
            AppError::NotFound(msg) => write!(f, "Not found: {}", msg),
            AppError::InternalError(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::ContentModerationError(err) => {
                let body = Json(json!({
                    "error": err.message,
                    "status": StatusCode::BAD_REQUEST.as_u16(),
                    "error_type": "content_moderation",
                    "violation_type": err.violation_type,
                    "details": err.details
                }));
                (StatusCode::BAD_REQUEST, body).into_response()
            },
            _ => {
                let (status, error_message) = match self {
                    AppError::DatabaseError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
                    AppError::ValidationError(msg) => (StatusCode::BAD_REQUEST, msg),
                    AppError::AuthError(msg) => (StatusCode::UNAUTHORIZED, msg),
                    AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
                    AppError::InternalError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
                    AppError::ContentModerationError(_) => unreachable!(), // Already handled above
                };

                let body = Json(json!({
                    "error": error_message,
                    "status": status.as_u16()
                }));

                (status, body).into_response()
            }
        }
    }
}

// Convenient Result type for the application
pub type Result<T> = std::result::Result<T, AppError>;