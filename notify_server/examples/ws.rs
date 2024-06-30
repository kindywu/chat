use axum::{
    extract::ws::{WebSocket, WebSocketUpgrade},
    response::Response,
    routing::get,
    Router,
};
use tracing::warn;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/ws", get(handler));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn handler(ws: WebSocketUpgrade) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    while let Some(msg) = socket.recv().await {
        let msg = match msg {
            Ok(msg) => msg,
            Err(e) => {
                warn!("client disconnected {e}");
                return;
            }
        };

        match socket.send(msg).await {
            Ok(_) => todo!(),
            Err(e) => {
                warn!("client disconnected {e}");
                return;
            }
        }
    }
}
