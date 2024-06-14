use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Extension, Json};
use hyper::StatusCode;

use crate::{services::User, AppError, AppState};

pub(crate) async fn list_chats_handler(
    Extension(user): Extension<User>,
    State(_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    Ok((StatusCode::OK, Json(user)))
}
