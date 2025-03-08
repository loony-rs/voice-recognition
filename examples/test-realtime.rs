use speechmatics::realtime::*;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};
use tokio::{self, fs::File, try_join};
use axum::{
    extract::ws::{Message, WebSocketUpgrade}, http::Response, response::IntoResponse, routing::get, Router
};
use tokio::net::TcpListener;
use futures_util::{StreamExt, SinkExt};

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
    env_logger::init();
    let app = Router::new()
    .route("/", get(home))
    .route("/ws", get(websocket_handler));
    
    let listener = TcpListener::bind("127.0.0.1:7000").await.unwrap();
    log::debug!("Listening on {}", "127.0.0.1:7000");
    axum::serve(listener, app).await.unwrap();
}

async fn home() -> impl IntoResponse {
    "Welcome!"
}

async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(|mut socket| async move {
        while let Some(Ok(msg)) = socket.next().await {
            match msg {
                Message::Text(utf8_bytes) => {
                    log::debug!("{}", utf8_bytes.as_str());
                },
                Message::Binary(bytes) => {},
                Message::Close(close_frame) => {},
                _ => {}
                // Message::Ping(bytes) => todo!(),
                // Message::Pong(bytes) => todo!(),
            }
            // if msg. || msg.is_binary() {
            //     socket.send(msg).await.expect("Failed to send message");
            // }
        }
    })
}


async fn translate() {
    let api_key: String = std::env::var("API_KEY").unwrap();
    let (mut rt_session, mut receive_channel) = RealtimeSession::new(api_key, None).unwrap();

    let test_file_path = PathBuf::new()
        .join(".")
        .join("tests")
        .join("data")
        .join("example.wav");

    let file = File::open(test_file_path).await.unwrap();

    let mut config: SessionConfig = Default::default();
    let audio_config = models::AudioFormat::new(models::audio_format::Type::File);
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

    let run_task = { rt_session.run(config, file) };

    try_join!(
        async move { message_task.await.map_err(anyhow::Error::from) },
        run_task
    )
    .unwrap();

    mock_store.lock().unwrap().print();


}
