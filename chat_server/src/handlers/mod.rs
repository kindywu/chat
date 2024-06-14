mod auth;

use std::sync::Arc;

use axum::{
    middleware::from_fn,
    routing::{get, post},
    Router,
};
use tower::ServiceBuilder;
use tower_http::{
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

use crate::{middlewares::set_request_id, AppState};

pub fn get_router(state: AppState) -> Router {
    let shared_app_state = Arc::new(state);

    let api = Router::new()
        .route("/signup", post(auth::signup_handler))
        .route("/signin", post(auth::signin_handler))
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().include_headers(true))
                        .on_request(DefaultOnRequest::new().level(Level::INFO))
                        .on_response(
                            DefaultOnResponse::new()
                                .level(Level::INFO)
                                .latency_unit(LatencyUnit::Micros),
                        ),
                )
                .layer(from_fn(set_request_id)),
        );

    Router::new()
        .route("/online", get(|| async { "chat sever online" }))
        .nest("/api", api)
        .with_state(shared_app_state)
}
