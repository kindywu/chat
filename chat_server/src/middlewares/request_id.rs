use axum::{extract::Request, http::HeaderValue, middleware::Next, response::Response};
use tracing::warn;

use super::REQUEST_ID_HEADER;
use uuid::Uuid;

pub async fn set_request_id(mut req: Request, next: Next) -> Response {
    let id = match req.headers().get(REQUEST_ID_HEADER) {
        Some(id) => Some(id.clone()),
        None => {
            let id = Uuid::now_v7().to_string();
            if let Ok(id) = HeaderValue::from_str(&id) {
                if let Err(e) = req.headers_mut().try_insert(REQUEST_ID_HEADER, id.clone()) {
                    warn!("set_request_id failed, value {id:?}, error {e}")
                }
                Some(id)
            } else {
                warn!("set_request_id failed, value {id}");
                None
            }
        }
    };
    let mut res = next.run(req).await;
    if let Some(id) = id {
        if let Err(e) = res.headers_mut().try_insert(REQUEST_ID_HEADER, id.clone()) {
            warn!("set_request_id failed, value {id:?}, error {e}")
        }
    }
    res
}

#[cfg(test)]
mod tests {
    use axum::{
        body::Body, extract::Request, middleware::from_fn, response::IntoResponse, routing::get,
        Router,
    };
    use hyper::StatusCode;
    use tower::ServiceExt;

    use crate::middlewares::{set_request_id, REQUEST_ID_HEADER};

    async fn handler(_req: Request) -> impl IntoResponse {
        (StatusCode::OK, "ok")
    }

    #[tokio::test]
    async fn set_request_id_should_work() -> anyhow::Result<()> {
        let app = Router::new()
            .route("/", get(handler))
            .layer(from_fn(set_request_id));

        let req = Request::builder().uri("/").body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::OK);
        assert!(res.headers().get(REQUEST_ID_HEADER).is_some());

        Ok(())
    }
}
