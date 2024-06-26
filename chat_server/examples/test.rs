use axum::{
    body::Body,
    extract::{Multipart, Request},
    routing::{get, post},
    Router,
};
use hyper::StatusCode;
use tower::ServiceExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/upload", post(upload));

    let req = Request::builder().uri("/").body(Body::empty())?;

    let res = app.clone().oneshot(req).await?;
    assert_eq!(res.status(), StatusCode::OK);

    Ok(())
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
