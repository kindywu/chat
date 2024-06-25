use std::sync::Arc;

use axum::{extract::State, response::IntoResponse, Extension, Json};
use hyper::StatusCode;
use serde_json::json;

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

    let chat_id: serde_json::Value = json!({
        "chat_id": chat_id,
    });

    Ok((StatusCode::CREATED, Json(chat_id)))
}

#[cfg(test)]
mod test {

    use std::{collections::HashSet, sync::Arc};

    use anyhow::Result;
    use axum::{extract::State, response::IntoResponse, Extension, Json};
    use http_body_util::BodyExt;
    use hyper::StatusCode;
    use serde_json::Value;

    use crate::{
        handlers::chat::create_chat_handler,
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
        let ret: Value = serde_json::from_slice(&body)?;
        let chat_id = ret.get("chat_id");
        assert!(chat_id.is_some());
        assert!(chat_id.unwrap().is_number());
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
            "create chat error: the chat members ([99]) is not exist"
        );
        Ok(())
    }
}
