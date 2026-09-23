use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("unauthorized")]
    Unauthorized,
    #[error("forbidden: {0}")]
    Forbidden(&'static str),
    #[error("conflict: {0}")]
    Conflict(&'static str),
    #[error("invalid request: {0}")]
    Invalid(&'static str),
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

pub type AppResult<T> = Result<T, AppError>;

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code) = match &self {
            Self::Unauthorized => (StatusCode::UNAUTHORIZED, "unauthorized"),
            Self::Forbidden(_) => (StatusCode::FORBIDDEN, "forbidden"),
            Self::Conflict(_) => (StatusCode::CONFLICT, "conflict"),
            Self::Invalid(_) => (StatusCode::BAD_REQUEST, "invalid_request"),
            Self::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal_error"),
        };
        if matches!(self, Self::Internal(_)) { tracing::error!(error = ?self, "request failed"); }
        (status, Json(json!({"error": code, "message": self.to_string()}))).into_response()
    }
}

impl From<sqlx::Error> for AppError {
    fn from(value: sqlx::Error) -> Self { Self::Internal(value.into()) }
}

