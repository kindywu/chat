mod auth;
mod chat;
mod message;
mod model;

use std::{sync::Arc, time::Duration};

use axum::{
    middleware::{from_fn, from_fn_with_state},
    routing::{get, post},
    Router,
};
use chat::{create_chat_handler, list_chats_handler};
use message::{file_handler, list_message_handler, upload_handler};
use tower::ServiceBuilder;

use tower_http::{
    classify::{ServerErrorsAsFailures, SharedClassifier},
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tower_http::{compression::CompressionLayer, timeout::TimeoutLayer};
use tracing::Level;

use crate::{
    middlewares::{set_request_id, verify_chat, verify_token, ServerTimeLayer},
    AppState,
};

pub fn get_router(state: AppState) -> Router {
    let shared_app_state = Arc::new(state);

    let api = Router::new()
        .route("/:id/messages", get(list_message_handler))
        .layer(from_fn_with_state(shared_app_state.clone(), verify_chat))
        .route("/chats", get(list_chats_handler).post(create_chat_handler))
        .route("/upload", post(upload_handler))
        .route("/files/:ws_id/*path", get(file_handler))
        .layer(from_fn_with_state(shared_app_state.clone(), verify_token))
        .route("/signup", post(auth::signup_handler))
        .route("/signin", post(auth::signin_handler))
        .layer(
            ServiceBuilder::new()
                .layer(build_trace_layer())
                .layer(build_compression_layer())
                .layer(from_fn(set_request_id))
                .layer(TimeoutLayer::new(Duration::from_secs(2)))
                .layer(ServerTimeLayer),
        );

    Router::new()
        .route("/online", get(|| async { "chat sever online" }))
        .nest("/api", api)
        .with_state(shared_app_state)
}

fn build_compression_layer() -> CompressionLayer {
    CompressionLayer::new()
        .gzip(true)
        .br(true)
        .deflate(true)
        .zstd(true)
}

fn build_trace_layer() -> TraceLayer<SharedClassifier<ServerErrorsAsFailures>> {
    TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().include_headers(true))
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(
            DefaultOnResponse::new()
                // .include_headers(true)
                .level(Level::INFO)
                .latency_unit(LatencyUnit::Micros),
        )
}
