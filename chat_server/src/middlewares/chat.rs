use std::sync::Arc;

use crate::{services::User, AppError, AppState};
use axum::{
    extract::{FromRequestParts, Path, Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};

pub async fn verify_chat(State(state): State<Arc<AppState>>, req: Request, next: Next) -> Response {
    let (mut parts, body) = req.into_parts();
    let Path(chat_id) = Path::<i64>::from_request_parts(&mut parts, &state)
        .await
        .unwrap();

    let user = parts.extensions.get::<User>().unwrap();
    println!("{},{}", chat_id, user.id);
    if !state
        .is_chat_member(chat_id, user.id)
        .await
        .unwrap_or_default()
    {
        let err = AppError::VerifyChat(format!(
            "User {} are not a member of chat {chat_id}",
            user.id
        ));
        return err.into_response();
    }

    let req = Request::from_parts(parts, body);
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use crate::middlewares::verify_token;

    use super::*;
    use anyhow::Result;
    use axum::{
        body::Body, http::StatusCode, middleware::from_fn_with_state, routing::get, Router,
    };
    use tower::ServiceExt;

    async fn handler(_req: Request) -> impl IntoResponse {
        (StatusCode::OK, "ok")
    }

    #[tokio::test]
    async fn verify_chat_middleware_should_work() -> Result<()> {
        let (_tdb, state) = AppState::try_new_test().await?;
        let state = Arc::new(state);

        let user = User::new(1, "Tyr Chen", "tchen@acme.org");
        let token = state.ek.sign(user)?;

        let app = Router::new()
            .route("/chat/:id/messages", get(handler))
            .layer(from_fn_with_state(state.clone(), verify_chat))
            .layer(from_fn_with_state(state.clone(), verify_token))
            .with_state(state);

        // user in chat
        let req = Request::builder()
            .uri("/chat/1/messages")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::OK);

        // user not in chat
        let req = Request::builder()
            .uri("/chat/5/messages")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())?;
        let res = app.oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::FORBIDDEN);

        Ok(())
    }
}
