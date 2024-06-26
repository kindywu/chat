use anyhow::Result;
use axum::extract::Json;
use axum::routing::put;
use axum::Router;
use axum_test::TestServer;
use hyper::StatusCode;
use serde_json::json;
use serde_json::Value;

async fn route_put_user(Json(user): Json<Value>) -> Json<Value> {
    Json(user)
}

#[tokio::main]
async fn main() -> Result<()> {
    let my_app = Router::new().route("/users", put(route_put_user));

    let server = TestServer::new(my_app)?;

    let response = server
        .put("/users")
        .json(&json!({
            "username": "Terrance Pencilworth",
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);
    Ok(())
}
