use std::sync::Arc;

use axum::{
    extract::{Multipart, Path, State},
    response::IntoResponse,
    Extension, Json,
};
use hyper::{HeaderMap, StatusCode};
use tokio::fs;
use tracing::warn;

use crate::{services::User, AppError, AppState};

use super::model::ChatFile;

pub(crate) async fn file_handler(
    Extension(user): Extension<User>,
    State(state): State<Arc<AppState>>,
    Path((ws_id, path)): Path<(i64, String)>,
) -> Result<impl IntoResponse, AppError> {
    if user.ws_id != ws_id {
        return Err(AppError::FileNotFound(
            "you don't have permission".to_string(),
        ));
    }

    let base_dir = state.file.base_dir.join(ws_id.to_string());
    let path = base_dir.join(path);
    if !path.exists() {
        return Err(AppError::FileNotFound("file doesn't exist".to_string()));
    }

    let mime = mime_guess::from_path(&path).first_or_octet_stream();
    let mut headers = HeaderMap::new();
    headers.insert("content-type", mime.to_string().parse().unwrap());

    let body = fs::read(&path).await?;
    Ok((headers, body))
}

pub(crate) async fn upload_handler(
    Extension(user): Extension<User>,
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let ws_id = user.ws_id;
    let base_dir = &state.file.base_dir;

    let mut urls = Vec::new();
    while let Some(field) = multipart.next_field().await? {
        // let Some(name) = field.name() else {
        //     warn!("multipart field name is not exist");
        //     continue;
        // };
        // if name != "fileupload" {
        //     warn!("multipart field name is not fileupload");
        //     continue;
        // }

        let Some(filename) = field.file_name() else {
            warn!("Failed to read filename from multipart field");
            continue;
        };
        let filename = filename.to_string();
        let Ok(data) = field.bytes().await else {
            warn!("Failed to read bytes from multipart field");
            continue;
        };

        let chat_file = ChatFile::new(ws_id, filename.as_str(), &data);
        let path = chat_file.path(base_dir);
        if path.exists() {
            warn!("File {} already exists: {:?}", filename, path);
        } else {
            fs::create_dir_all(path.parent().expect("file path parent should exists")).await?;
            fs::write(path, data).await?;
        }
        urls.push(chat_file.url())
    }

    Ok((StatusCode::OK, Json(urls)))
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use anyhow::Result;
    use axum::{
        extract::{Path, State},
        middleware::from_fn_with_state,
        response::IntoResponse,
        routing::post,
        Extension, Router,
    };
    use axum_test::{
        multipart::{MultipartForm, Part},
        TestServer,
    };
    use http_body_util::BodyExt;
    use hyper::StatusCode;
    use serde_json::json;

    use crate::{
        handlers::{
            auth::{signin_handler, AuthOutput},
            message::{file_handler, upload_handler},
        },
        middlewares::verify_token,
        services::User,
        AppState,
    };

    #[tokio::test]
    async fn upload_handler_should_work() -> Result<()> {
        let (_tdb, state) = AppState::try_new_test().await?;

        let shared_app_state = Arc::new(state);
        let app = Router::new()
            .route("/upload", post(upload_handler))
            .layer(from_fn_with_state(shared_app_state.clone(), verify_token))
            .route("/signin", post(signin_handler))
            .with_state(shared_app_state);

        let server = TestServer::new(app)?;

        let signin = json!({
            "email": "tchen@acme.org","password": "123456"
        });
        let response = server.post("/signin").json(&signin).await;
        assert_eq!(response.status_code(), StatusCode::OK);

        let body = response.as_bytes();
        let auth: AuthOutput = serde_json::from_slice(body)?;

        let image_bytes = include_bytes!("../../assets/demo.jpg");
        let image_part = Part::bytes(image_bytes.as_slice())
            .file_name("demo.jpg")
            .mime_type("image/jpg");

        let form = MultipartForm::new().add_part("file", image_part);

        let response = server
            .post("/upload")
            .multipart(form)
            .authorization_bearer(auth.token)
            .await;

        assert_eq!(response.status_code(), StatusCode::OK);
        let body = response.as_bytes();
        let files: Vec<&str> = serde_json::from_slice(body)?;
        assert_eq!(files.len(), 1);
        assert_eq!(
            files[0],
            "/files/1/8ab/d00/a3253d525b37958381ba1cb044d1cad887.jpg"
        );

        Ok(())
    }

    #[tokio::test]
    async fn file_handler_should_work() -> Result<()> {
        let (_tdb, state) = AppState::try_new_test().await?;
        let user = User {
            ws_id: 1,
            ..Default::default()
        };

        let ws_id = 1;
        let path = "0a0/a9f/2a6772942557ab5355d76af442f8f65e01.txt".to_string();
        let ret = file_handler(Extension(user), State(Arc::new(state)), Path((ws_id, path)))
            .await?
            .into_response();

        assert_eq!(ret.status(), StatusCode::OK);
        let body = ret.into_body().collect().await?.to_bytes().to_vec();
        assert_eq!(body, b"Hello, World!");
        Ok(())
    }
}
