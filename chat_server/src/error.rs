use std::io;

use axum::{extract::multipart::MultipartError, http::StatusCode, response::IntoResponse, Json};
use hyper::header::InvalidHeaderValue;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("email not found: {0}")]
    EmailNotFound(String),

    #[error("email already exists: {0}")]
    EmailAlreadyExists(String),

    #[error("file not found: {0}")]
    FileNotFound(String),

    #[error("io error: {0}")]
    IoError(#[from] io::Error),

    #[error("invalid header value error: {0}")]
    InvalidHeaderValue(#[from] InvalidHeaderValue),

    #[error("multipart error: {0}")]
    MultipartError(#[from] MultipartError),

    #[error("sql error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("password hash error: {0}")]
    PasswordHashError(#[from] argon2::password_hash::Error),

    #[error("jwt sign hash error: {0}")]
    SignError(#[from] jwt_simple::Error),

    #[error("create chat error: {0}")]
    CreateChatError(String),

    #[error("chat file error: {0}")]
    ChatFileError(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorOutput {
    pub error: String,
}

impl ErrorOutput {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            AppError::EmailNotFound(_) => StatusCode::NOT_FOUND,
            AppError::EmailAlreadyExists(_) => StatusCode::CONFLICT,
            AppError::PasswordHashError(_) => StatusCode::FORBIDDEN,
            AppError::SignError(_) => StatusCode::FORBIDDEN,
            AppError::CreateChatError(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let err_output = ErrorOutput {
            error: self.to_string(),
        };

        (status, Json(err_output)).into_response()
    }
}
