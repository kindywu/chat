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
