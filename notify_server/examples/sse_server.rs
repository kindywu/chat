use std::{convert::Infallible, sync::Arc};

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
use futures::stream::Stream;
use tokio::sync::broadcast;

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
    let mut rx = app_state.broadcaster.add_client(&name).await;

    struct Guard {
        name: String,
        broadcaster: Arc<Broadcaster>,
    }

    impl Drop for Guard {
        fn drop(&mut self) {
            println!("stream closed");
            self.broadcaster.remove_client(&self.name);
        }
    }

    let stream = async_stream::stream! {
        let _guard = Guard {
            name,
            broadcaster:app_state.broadcaster.clone()
        };
        while let Ok(msg) = rx.recv().await{
            yield Ok(Event::default().data(msg));
        }
        println!("`_guard` is dropped")
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

// async fn sse_handler() -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
//     struct Guard {
//         // whatever state you need here
//     }

//     impl Drop for Guard {
//         fn drop(&mut self) {
//             println!("stream closed");
//         }
//     }

//     let stream = async_stream::stream! {
//         let _guard = Guard {};
//         let mut interval = tokio::time::interval(std::time::Duration::from_secs(1));
//         loop {
//             interval.tick().await;
//             yield Ok(Event::default().data("hi"));
//         }
//         // `_guard` is dropped
//     };

//     Sse::new(stream).keep_alive(KeepAlive::default())
// }

struct BroadcasterInner {
    clients: DashMap<String, broadcast::Sender<String>>,
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

    pub async fn add_client(&self, name: &str) -> broadcast::Receiver<String> {
        println!("add_client {name}");
        let (tx, rx) = broadcast::channel::<String>(10);

        tx.send(format!("welcome to {name}")).unwrap();

        self.inner.clients.insert(name.to_owned(), tx.clone());

        rx
    }

    pub fn remove_client(&self, name: &str) {
        println!("remove_client {name}");
        self.inner.clients.remove(name);
    }

    pub async fn broadcast(&self, event: &str) {
        let clients = self.inner.clients.clone();

        println!("number of clients before broadcast: {}", clients.len());
        let clients: Vec<_> = clients.iter().map(|kv| kv.value().clone()).collect();

        let senders: Vec<_> = clients
            .iter()
            .map(|client| client.send(event.to_string()))
            .collect();

        println!(
            "number of clients after broadcast: {}",
            senders.iter().filter(|f| f.is_ok()).count()
        );
    }
}
