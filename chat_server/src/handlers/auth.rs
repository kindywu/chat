use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{AppError, AppState, CreateUser};

pub(crate) async fn signup_handler(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.create_user(&input).await?;
    let token = state.ek.sign(user)?;
    let body = Json(AuthOutput { token });
    Ok((StatusCode::CREATED, body))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthOutput {
    token: String,
}
