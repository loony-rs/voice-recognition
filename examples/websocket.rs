use axum::{
    extract::WebSocketUpgrade,
    response::IntoResponse,
    routing::get,
    Router,
};
use tokio::net::TcpListener;
use futures_util::{StreamExt, SinkExt};
use axum::extract::ws::{WebSocket, Message};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

  // build our application with a single route
  let app = Router::new().route("/", get(websocket_handler));

  // run our app with hyper, listening globally on port 3000
  let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
  axum::serve(listener, app).await.unwrap();
}

/// WebSocket handler
async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

/// Handles the WebSocket connection
async fn handle_socket(mut socket: WebSocket) {
    tracing::info!("New WebSocket connection");

    while let Some(Ok(msg)) = socket.recv().await {
        match msg {
            Message::Text(text) => {
                tracing::info!("Received: {}", text);
            }
            Message::Close(_) => {
                tracing::info!("WebSocket closed");
                break;
            }
            _ => {}
        }
    }
}
