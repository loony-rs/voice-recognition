use std::sync::Arc;

use axum::{
    extract::WebSocketUpgrade,
    response::IntoResponse,
    routing::get,
    Router,
    extract::State
};
use axum::extract::ws::WebSocket;
use futures::stream::{SplitStream, SplitSink};
use voice_recognition::realtime::models::{self, EndOfStream, StartRecognition};
use voice_recognition::realtime::ReadMessage;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{tungstenite, MaybeTlsStream, WebSocketStream};
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use url::Url;
use base64::{engine::general_purpose, Engine as _};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use voice_recognition::config::{get_audio_format, get_transcription_config};

type SpeechmaticsSender = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type SpeechmaticsReceiver = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

struct AppState {
    api_key: String
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let api_key = std::env::var("API_KEY").unwrap();
    let port = std::env::var("PORT").unwrap_or("2000".to_string());
    let port = port.parse::<i32>().unwrap();
    let app_state = AppState {
        api_key
    };
    
    let app = Router::new().route("/", get(websocket_handler)).with_state(Arc::new(app_state));
    let url = format!("127.0.0.1:{}", port);
    
    let listener = tokio::net::TcpListener::bind(&url).await.unwrap();
    log::info!("Listening on {}", url);
    axum::serve(listener, app).await.unwrap();
}

struct SpeechmaticsReceiverDrop;

impl Drop for SpeechmaticsReceiverDrop {
    fn drop(&mut self) {
        log::info!("SpeechmaticsReceiverDropped");
    }
}

/// WebSocket handler
async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    ws.on_upgrade(|socket| async {
        handle_socket(socket, state).await;
    })
}

/// Handles the WebSocket connection
async fn handle_socket(socket: WebSocket, state: Arc<AppState>) {
    let ( _, mut receiver) = socket.split();
    let (mut speechmatics_sender, mut speechmatics_receiver) = connect_speechmatics(state.api_key.clone()).await.unwrap();
    let start_recognition_msg = start_recognition_msg().unwrap();

    let handle1 = tokio::spawn(async move {
        let _ = SpeechmaticsReceiverDrop;
        let mut last_seq_no = 0;
        while let Some(data) = receiver.next().await {
            if let Ok(data) = data {
                match data {
                    axum::extract::ws::Message::Text(utf8_bytes) => {
                        if utf8_bytes.as_str() == "START_VOICE_RECORDING" {
                            speechmatics_sender.send(tungstenite::Message::Text(start_recognition_msg.to_string())).await.unwrap();
                        }
                        if utf8_bytes.as_str() == "STOP_VOICE_RECORDING" {
                            // let close_msg = end_stream_msg(last_seq_no).unwrap();
                            // speechmatics_sender.send(tungstenite::Message::Text(close_msg.to_string())).await.unwrap();
                        }
                    },
                    axum::extract::ws::Message::Binary(bytes) => {
                        speechmatics_sender.send(tungstenite::Message::binary(bytes.to_vec())).await.unwrap();
                        last_seq_no += 1;
                        
                    },
                    _ => {}
                }
            }
        }
      });


    tokio::spawn(async move {
        let _ = SpeechmaticsReceiverDrop;
        loop {
            let value = speechmatics_receiver.next().await;
            if let Some(value) = value {
                match value {
                    Ok(msg) => {
                        println!("{:?}", msg);
                        let data = msg.into_data();
                        let data = serde_json::from_slice::<ReadMessage>(&data);
                        if let Ok(data) = data {
                            match data {
                                ReadMessage::RecognitionStarted(_) => {
                                    log::info!("RecognitionStarted");
                                },
                                ReadMessage::Error(error) => {
                                    log::error!("Error: {:?}", error);
                                },
                                ReadMessage::AddTranscript(message) => {
                                    log::info!("{:?}", message.metadata.transcript);
                                },
                                ReadMessage::EndOfTranscript(_) => {
                                    log::info!("EndOfTranscript");
                                    handle1.abort();
                                    break;
                                },
                                _ => {}
                            }
                        }
                    },
                    Err(err) => {
                        log::error!("{:?}", err);
                    },
                }
            }
        }
    });

}

async fn connect_speechmatics(api_key: String) -> std::result::Result<(SpeechmaticsSender, SpeechmaticsReceiver), ()> {
    let url = format!("wss://eu2.rt.speechmatics.com/v2?jwt={}", api_key);
    let authorization = format!("Bearer {}", api_key); 
    let url = Url::parse(&url).unwrap();
    let sec_key: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();
    let b64 = general_purpose::STANDARD.encode(sec_key);

    let request = http::Request::builder()
        .method("GET")
        .uri(url.as_str())
        .header("Host", "eu2.rt.speechmatics.com")
        .header("Authorization", &authorization)
        .header("Sec-WebSocket-Key", b64)
        .header("Connection", "keep-alive, Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .body(())
        .unwrap();

    let (speechmatics_stream, _) = connect_async(request).await.expect("Failed to connect");
    Ok(speechmatics_stream.split())
}

fn start_recognition_msg() -> anyhow::Result<Message> {
    let message: models::StartRecognition = StartRecognition::new(
        get_audio_format(), 
        models::start_recognition::Message::StartRecognition, 
        get_transcription_config()
    );
    let serialised_msg = serde_json::to_string(&message)?;
    let ws_message = Message::from(serialised_msg);
    Ok(ws_message)
}

fn end_stream_msg(last_seq_no: i32) -> anyhow::Result<Message> {
    let message = EndOfStream {
        last_seq_no,
        message: models::end_of_stream::Message::EndOfStream,
    };
    let serialised_msg = serde_json::to_string(&message)?;
    let ws_message = Message::from(serialised_msg);
    Ok(ws_message)
}