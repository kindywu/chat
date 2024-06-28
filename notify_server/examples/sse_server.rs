use axum::{
    debug_handler,
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
    routing::get,
    Router,
};
use chrono::Local;
use dashmap::DashMap;
use futures_util::stream::Stream;
use nanoid::nanoid;
use std::{convert::Infallible, ops::Deref, sync::Arc, time::Duration};
use tokio::sync::mpsc::{self, Sender};
use tracing::warn;

#[derive(Clone, Debug)]
struct AppState {
    map: Arc<DashMap<String, Sender<String>>>,
}

impl AppState {
    fn new() -> Self {
        Self {
            map: Arc::new(DashMap::new()),
        }
    }
}

impl Deref for AppState {
    type Target = DashMap<String, Sender<String>>;

    fn deref(&self) -> &Self::Target {
        &self.map
    }
}

#[tokio::main]
async fn main() {
    let app_state = AppState::new();

    let task_state = app_state.clone();
    // 启动一个任务来发送数据到 mpsc 通道
    tokio::spawn(async move {
        loop {
            for entry in task_state.map.iter() {
                let sender = entry.value(); // 克隆 Sender
                let name = entry.key(); // 克隆 key

                let msg = format!("hello {name}. now is {:?}", Local::now());
                if let Err(e) = sender.send(msg).await {
                    warn!("error happen: {e}, {name} quit");
                }
            }

            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    });

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/sse", get(sse_handler))
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[debug_handler]
async fn sse_handler(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let (tx, mut rx) = mpsc::channel::<String>(100);

    loop {
        let id = nanoid!(10);
        if state.map.insert(id.clone(), tx.clone()).is_none() {
            break;
        }
    }

    let stream = async_stream::stream! {
        while let Some(message) = rx.recv().await {
            yield Ok(Event::default().data(message));
        }
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}
