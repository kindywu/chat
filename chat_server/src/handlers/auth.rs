use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{services::SigninUser, AppError, AppState, CreateUser, ErrorOutput};

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
            let body = Json(ErrorOutput::new("Invalid email or password"));
            Ok((StatusCode::FORBIDDEN, body).into_response())
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthOutput {
    token: String,
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use anyhow::Result;
    use axum::{body::to_bytes, extract::State, http::StatusCode, response::IntoResponse, Json};
    use jwt_simple::reexports::serde_json;

    use crate::{
        handlers::auth::{signup_handler, AuthOutput},
        AppState, CreateUser, ErrorOutput,
    };

    #[tokio::test]
    async fn signup_should_work() -> Result<()> {
        let (_tdb, state) = AppState::try_new_test().await?;
        let input = CreateUser::new("acme", "Tian Chen", "tyr@acme.org", "123456");
        let ret = signup_handler(State(Arc::new(state)), Json(input))
            .await?
            .into_response();
        assert_eq!(ret.status(), StatusCode::CREATED);
        let body = ret.into_body();
        let body = to_bytes(body, usize::MAX).await?;
        // let body = ret.into_body().collect().await?.to_bytes();
        let ret: AuthOutput = serde_json::from_slice(&body)?;
        assert_ne!(ret.token, "");
        Ok(())
    }

    #[tokio::test]
    async fn signup_duplicate_user_should_409() -> Result<()> {
        let (_tdb, state) = AppState::try_new_test().await?;
        let input = CreateUser::new("acme", "Tyr Chen", "tchen@acme.org", "123456");

        let ret = signup_handler(State(Arc::new(state)), Json(input))
            .await
            .into_response();
        assert_eq!(ret.status(), StatusCode::CONFLICT);
        let body = ret.into_body();
        let body = to_bytes(body, usize::MAX).await?;
        // let body = body.collect().await?.to_bytes();
        let ret: ErrorOutput = serde_json::from_slice(&body)?;

        assert_eq!(ret.error, "email already exists: tchen@acme.org");
        Ok(())
    }
}
