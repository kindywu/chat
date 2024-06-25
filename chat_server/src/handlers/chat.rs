use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    services::{CreateChat, User},
    AppError, AppState,
};

pub(crate) async fn list_chats_handler(
    Extension(user): Extension<User>,
    State(_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AppError> {
    Ok((StatusCode::OK, Json(user)))
}

pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<Arc<AppState>>,
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat_id = state.create_chat(&input, user.ws_id).await?;
    Ok((StatusCode::CREATED, Json(CreateChatOutput { chat_id })))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateChatOutput {
    chat_id: i64,
}

#[cfg(test)]
mod test {

    use std::{collections::HashSet, sync::Arc};

    use anyhow::Result;
    use axum::{extract::State, response::IntoResponse, Extension, Json};
    use http_body_util::BodyExt;
    use hyper::StatusCode;

    use crate::{
        handlers::chat::{create_chat_handler, CreateChatOutput},
        services::{CreateChat, User},
        AppState, ErrorOutput,
    };

    #[tokio::test]
    async fn create_chat_handler_should_work() -> Result<()> {
        let (_tdb, state) = AppState::try_new_test().await?;
        let user = User {
            ws_id: 1,
            ..Default::default()
        };
        let input = CreateChat::new(None, HashSet::from([1, 2, 3]), true);
        let ret = create_chat_handler(Extension(user), State(Arc::new(state)), Json(input))
            .await?
            .into_response();
        assert_eq!(ret.status(), StatusCode::CREATED);
        // let body = ret.into_body();
        // let body = to_bytes(body, usize::MAX).await?;
        let body = ret.into_body().collect().await?.to_bytes();
        let ret: CreateChatOutput = serde_json::from_slice(&body)?;
        assert!(ret.chat_id.is_positive());
        Ok(())
    }

    #[tokio::test]
    async fn create_chat_handler_should_fail_because_99() -> Result<()> {
        let (_tdb, state) = AppState::try_new_test().await?;
        let user = User {
            ws_id: 1,
            ..Default::default()
        };
        let input = CreateChat::new(None, HashSet::from([1, 2, 99]), true);
        let ret = create_chat_handler(Extension(user), State(Arc::new(state)), Json(input)).await;
        let ret = ret.into_response();
        assert_eq!(ret.status(), StatusCode::BAD_REQUEST);

        let body = ret.into_body().collect().await?.to_bytes();
        let ret: ErrorOutput = serde_json::from_slice(&body)?;
        assert_eq!(
            ret.error,
            "create chat with error: the chat members ([99]) is not exist"
        );
        Ok(())
    }
}
