use axum::{http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("email not found: {0}")]
    EmailNotFound(String),

    #[error("email already exists: {0}")]
    EmailAlreadyExists(String),

    #[error("sql error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("password hash error: {0}")]
    PasswordHashError(#[from] argon2::password_hash::Error),

    #[error("jwt sign hash error: {0}")]
    SignError(#[from] jwt_simple::Error),
}

#[derive(Debug, Serialize)]
pub struct ErrorOutput {
    error: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            AppError::EmailAlreadyExists(_) => StatusCode::CONFLICT,
            _ => unimplemented!(),
        };

        let err_output = ErrorOutput {
            error: self.to_string(),
        };

        (status, Json(err_output)).into_response()
    }
}
