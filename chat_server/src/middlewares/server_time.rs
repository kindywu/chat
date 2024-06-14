use axum::{extract::Request, http::HeaderValue, response::Response};
use futures_util::future::BoxFuture;
use std::task::{Context, Poll};
use tokio::time::Instant;
use tower::{Layer, Service};
use tracing::warn;

use super::SERVER_TIME_HEADER;

#[derive(Clone)]
pub struct ServerTimeLayer;

impl<S> Layer<S> for ServerTimeLayer {
    type Service = ServerTimeMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ServerTimeMiddleware { inner }
    }
}

#[derive(Clone)]
pub struct ServerTimeMiddleware<S> {
    inner: S,
}

impl<S> Service<Request> for ServerTimeMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let start = Instant::now();
        let future = self.inner.call(request);
        Box::pin(async move {
            let mut res: Response = future.await?;
            let elapsed = format!("{}us", start.elapsed().as_micros());
            if let Ok(elapsed) = HeaderValue::from_str(&elapsed) {
                if let Err(e) = res
                    .headers_mut()
                    .try_insert(SERVER_TIME_HEADER, elapsed.clone())
                {
                    warn!("set_server_time failed, value {elapsed:?}, error {e}")
                }
            }
            Ok(res)
        })
    }
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, extract::Request, routing::get, Router};
    use hyper::StatusCode;
    use tower::ServiceExt;

    use crate::middlewares::{test_handler, ServerTimeLayer, SERVER_TIME_HEADER};

    #[tokio::test]
    async fn server_time_should_work() -> anyhow::Result<()> {
        let app = Router::new()
            .route("/", get(test_handler))
            .layer(ServerTimeLayer);

        let req = Request::builder().uri("/").body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::OK);
        assert!(res.headers().get(SERVER_TIME_HEADER).is_some());

        Ok(())
    }
}
