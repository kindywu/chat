use axum::{
    extract::State,
    response::{
        sse::{Event, KeepAlive, Sse},
        Html,
    },
    routing::get,
    Router,
};
use dashmap::DashMap;
use futures::{future, stream::Stream};
use std::{convert::Infallible, sync::Arc};
use tokio::sync::mpsc;
use tokio_stream::{wrappers::ReceiverStream, StreamExt};

#[tokio::main]
async fn main() {
    let broadcaster = Broadcaster::new();

    let app_state = Arc::new(AppState { broadcaster });

    let html = r#"
        Open Console
        <script>
            const es = new EventSource("http://localhost:3000/sse");
            es.onopen = () => console.log("Connection Open!");
            es.onmessage = (e) => console.log("Message:", e);
            es.onerror = (e) => {
                console.log("Error:", e);
                // es.close();
            };
        </script>
        "#;
    let html = html.to_string();

    let app = Router::new()
        .route("/", get(|| async { Html::from(html) }))
        .route("/send_message", get(send_message))
        .route("/sse", get(sse_handler))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

async fn send_message(State(app_state): State<Arc<AppState>>) -> Html<&'static str> {
    app_state.broadcaster.broadcast("message").await;
    Html("Message sent")
}

async fn sse_handler(
    State(app_state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let name = nanoid::nanoid!(8);
    let (tx, rx) = app_state.broadcaster.add_client(&name).await;

    app_state
        .broadcaster
        .handle_client_disconnected(name.to_string(), tx);

    let my_stream = ReceiverStream::<String>::new(rx).map(|res| Ok(Event::default().data(res)));
    Sse::new(my_stream).keep_alive(KeepAlive::default())
}

struct BroadcasterInner {
    clients: DashMap<String, mpsc::Sender<String>>,
}

pub struct Broadcaster {
    inner: Arc<BroadcasterInner>,
}

struct AppState {
    broadcaster: Arc<Broadcaster>,
}

impl Broadcaster {
    pub fn new() -> Arc<Self> {
        Arc::new(Broadcaster {
            inner: Arc::new(BroadcasterInner {
                clients: DashMap::new(),
            }),
        })
    }

    pub async fn add_client(&self, name: &str) -> (mpsc::Sender<String>, mpsc::Receiver<String>) {
        let (tx, rx) = mpsc::channel::<String>(10);

        tx.send(format!("welcome to {name}")).await.unwrap();

        self.inner.clients.insert(name.to_owned(), tx.clone());

        (tx, rx)
    }

    pub fn handle_client_disconnected(&self, name: String, tx: mpsc::Sender<String>) {
        let inner = self.inner.clone();
        tokio::spawn(async move {
            tx.closed().await;
            inner.clients.remove(&name);
            println!(
                "number of clients after handle_client_disconnected {}",
                inner.clients.len()
            );
        });
    }

    pub async fn broadcast(&self, event: &str) {
        let clients = self.inner.clients.clone();

        println!("number of clients before broadcast: {}", clients.len());
        let clients: Vec<_> = clients.iter().map(|kv| kv.value().clone()).collect();

        let send_futures = clients.iter().map(|client| client.send(event.to_string()));

        let results = future::join_all(send_futures).await;
        println!(
            "number of clients after broadcast: {}",
            results.iter().filter(|f| f.is_ok()).count()
        );
    }
}
