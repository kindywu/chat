mod auth;

use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use crate::AppState;

pub fn get_router(state: AppState) -> Router {
    let shared_app_state = Arc::new(state);

    let api = Router::new()
        .route("/signup", post(auth::signup_handler))
        .route("/signin", post(auth::signin_handler));

    Router::new()
        .route("/online", get(|| async { "chat sever online" }))
        .nest("/api", api)
        .with_state(shared_app_state)
}
