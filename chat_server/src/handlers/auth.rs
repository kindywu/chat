use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{services::SigninUser, AppError, AppState, CreateUser};

pub(crate) async fn signup_handler(
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.create_user(&input).await?;
    let token = state.ek.sign(user)?;
    let body = Json(AuthOutput { token });
    Ok((StatusCode::CREATED, body))
}

pub(crate) async fn signin_handler(
    State(state): State<Arc<AppState>>,
    Json(input): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = state.verify_user(&input).await?;
    match user {
        Some(user) => {
            let token = state.ek.sign(user)?;
            Ok((StatusCode::OK, Json(AuthOutput { token })).into_response())
        }
        None => {
            let body = Json(ErrorOutput {
                error: "Invalid email or password".into(),
            });
            Ok((StatusCode::FORBIDDEN, body).into_response())
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthOutput {
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorOutput {
    error: String,
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    #[tokio::test]
    async fn find_user_by_noexist_email_should_work() -> Result<()> {
        Ok(())
    }

    #[tokio::test]
    async fn find_user_by_email_should_work() -> Result<()> {
        Ok(())
    }
}
