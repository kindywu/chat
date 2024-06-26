use anyhow::Result;
use axum::extract::Json;
use axum::extract::Multipart;
use axum::routing::post;
use axum::routing::put;
use axum::Router;
use axum_test::multipart::MultipartForm;
use axum_test::multipart::Part;
use axum_test::TestServer;
use hyper::StatusCode;
use serde_json::json;
use serde_json::Value;

async fn route_put_user(Json(user): Json<Value>) -> Json<Value> {
    Json(user)
}

async fn upload(mut multipart: Multipart) {
    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap().to_string();
        let file_name = field.file_name().unwrap_or_default().to_owned();
        let data = field.bytes().await.unwrap();

        println!(
            "Length of `{}` is {} bytes, filename is {}",
            file_name,
            data.len(),
            name,
        );
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let my_app = Router::new()
        .route("/users", put(route_put_user))
        .route("/upload", post(upload));

    let server = TestServer::new(my_app)?;

    let response = server
        .put("/users")
        .json(&json!({
            "username": "Terrance Pencilworth",
        }))
        .await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let image_bytes = include_bytes!("../assets/demo.jpg");
    let image_part = Part::bytes(image_bytes.as_slice())
        .file_name("demo.jpg")
        .mime_type("image/jpg");

    let form = MultipartForm::new().add_part("file", image_part);

    let response = server.post("/upload").multipart(form).await;

    assert_eq!(response.status_code(), StatusCode::OK);

    Ok(())
}
