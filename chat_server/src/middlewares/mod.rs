mod auth;
mod request_id;
mod server_time;

pub use auth::verify_token;
pub use request_id::set_request_id;
pub use server_time::ServerTimeLayer;

const REQUEST_ID_HEADER: &str = "x-request-id";
const SERVER_TIME_HEADER: &str = "x-server-time";

#[cfg(test)]
async fn test_handler(_req: axum::extract::Request) -> impl axum::response::IntoResponse {
    (hyper::StatusCode::OK, "test ok")
}
