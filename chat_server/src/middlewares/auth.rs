use std::sync::Arc;

use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_auth::AuthBearer;
use tracing::warn;

use crate::AppState;

pub async fn verify_token(
    AuthBearer(token): AuthBearer,
    State(state): State<Arc<AppState>>,
    mut request: Request,
    next: Next,
) -> Response {
    match state.dk.verify(&token) {
        Ok(user) => {
            request.extensions_mut().insert(user);
            next.run(request).await
        }
        Err(e) => {
            let msg = format!("verify token failed: {:?}", e);
            warn!(msg);
            (StatusCode::FORBIDDEN, msg).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::Body, extract::Request, middleware::from_fn_with_state, routing::get, Router,
    };
    use hyper::StatusCode;
    use tower::ServiceExt;

    use crate::{
        middlewares::{test_handler, verify_token},
        services::User,
        AppState,
    };

    #[tokio::test]
    async fn verify_token_should_work() -> anyhow::Result<()> {
        let (_tdb, state) = AppState::try_new_test().await?;
        let user = User::new(1, "Tyr Chen", "tchen@acme.org");
        let token = state.ek.sign(user)?;

        let state = Arc::new(state);

        let app = Router::new()
            .route("/", get(test_handler))
            .layer(from_fn_with_state(state.clone(), verify_token))
            .with_state(state);

        // good token
        let req = Request::builder()
            .uri("/")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::OK);

        // no token
        let req = Request::builder().uri("/").body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);

        // bad token
        let req = Request::builder()
            .uri("/")
            .header("Authorization", "Bearer bad-token")
            .body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
        Ok(())
    }
}
