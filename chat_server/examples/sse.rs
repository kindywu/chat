use axum::{
    debug_handler,
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
    Router,
};
use futures_util::stream::{self, Stream};
use std::{convert::Infallible, sync::Arc, time::Duration};
use tokio::{
    sync::{mpsc, Mutex},
    time::Instant,
};
use tokio_stream::StreamExt as _;

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel::<String>(100);
    let rx = Arc::new(Mutex::new(rx));

    // 启动一个任务来发送数据到 mpsc 通道
    tokio::spawn(async move {
        loop {
            tx.send(format!("hello from mpsc. {:?}", Instant::now()))
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    });

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/sse", get(sse_handler))
        .route("/sse_mpsc", get(move || sse_handler_mpsc(rx)));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[debug_handler]
async fn sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // A `Stream` that repeats an event every second
    let stream = stream::repeat_with(|| Event::default().data("hi!"))
        .map(Ok)
        .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn sse_handler_mpsc(
    rx: Arc<Mutex<mpsc::Receiver<String>>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        let mut rx = rx.lock().await;
        while let Some(message) = rx.recv().await {
            yield Ok(Event::default().data(message));
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}
