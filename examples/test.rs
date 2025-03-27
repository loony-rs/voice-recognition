use speechmatics::realtime::*;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::{self, fs::File, try_join};
use axum::{
    extract::WebSocketUpgrade,
    response::IntoResponse,
    routing::get,
    Router,
};
use axum::extract::ws::WebSocket;
use futures::{SinkExt, StreamExt};

struct MockStore {
    transcript: String,
}

impl MockStore {
    pub fn new() -> Self {
        Self {
            transcript: "".to_owned(),
        }
    }

    pub fn append(&mut self, transcript: String) {
        self.transcript = format!("{} {}", self.transcript, transcript);
    }

    pub fn print(&self) {
        print!("{}", self.transcript)
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

  // build our application with a single route
  let app = Router::new().route("/", get(websocket_handler));

  // run our app with hyper, listening globally on port 3000
  let listener = tokio::net::TcpListener::bind("localhost:2000").await.unwrap();
  log::info!("Listening on localhost:2000");
  axum::serve(listener, app).await.unwrap();
}

/// WebSocket handler
async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(socket: WebSocket) {
    let ( _, mut receiver) = socket.split();

    let api_key: String = "evK20Lpk7TTRtpNAv0Cbh4pCBzvr32Y6".to_string();
    let (mut rt_session, mut receive_channel) = RealtimeSession::new(api_key, Some("wss://eu2.rt.speechmatics.com/v2?jwt=evK20Lpk7TTRtpNAv0Cbh4pCBzvr32Y6".to_string())).unwrap();


    let mut config: SessionConfig = Default::default();
    let audio_config = models::AudioFormat::new(models::audio_format::Type::Raw);
    config.audio_format = Some(audio_config);

    let mock_store = Arc::new(Mutex::new(MockStore::new()));
    let mock_store_clone = mock_store.clone();

    let message_task = tokio::spawn(async move {
        while let Some(message) = receive_channel.recv().await {
            match message {
                ReadMessage::AddTranscript(mess) => {
                    mock_store_clone
                        .lock()
                        .unwrap()
                        .append(mess.metadata.transcript);
                }
                ReadMessage::EndOfTranscript(_) => return,
                _ => {}
            }
        }
    });

    let run_task = { rt_session.test_run(config, &mut receiver) };

    try_join!(
        async move { message_task.await.map_err(anyhow::Error::from) },
        run_task
    )
    .unwrap();

    mock_store.lock().unwrap().print();
}
