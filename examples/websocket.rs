use axum::{
    extract::WebSocketUpgrade,
    response::IntoResponse,
    routing::get,
    Router,
};
use axum::extract::ws::WebSocket;
use futures::stream::{SplitStream, SplitSink};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{tungstenite, MaybeTlsStream, WebSocketStream};
use futures::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use url::Url;
use base64::{engine::general_purpose, Engine as _};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

type SpeechmaticsSender = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>;
type SpeechmaticsReceiver = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

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

struct SpeechmaticsReceiverDrop;

impl Drop for SpeechmaticsReceiverDrop {
    fn drop(&mut self) {
        println!("Task memory was dropped");
    }
}


async fn connect_speechmatics() -> std::result::Result<(SpeechmaticsSender, SpeechmaticsReceiver), ()> {
    let url = Url::parse("wss://eu2.rt.speechmatics.com/v2?jwt=evK20Lpk7TTRtpNAv0Cbh4pCBzvr32Y6").unwrap();
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
        .header("Authorization", "Bearer evK20Lpk7TTRtpNAv0Cbh4pCBzvr32Y6")
        .header("Sec-WebSocket-Key", b64)
        .header("Connection", "keep-alive, Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .body(())
        .unwrap();

    let (speechmatics_stream, _) = connect_async(request).await.expect("Failed to connect");
    Ok(speechmatics_stream.split())
}

/// WebSocket handler
async fn websocket_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

/// Handles the WebSocket connection
async fn handle_socket(socket: WebSocket) {
    let ( _, mut receiver) = socket.split();
    let (mut speechmatics_sender, mut speechmatics_receiver) = connect_speechmatics().await.unwrap();

    let msg = serde_json::json!({
        "message": "StartRecognition",
        "audio_format": {
          "type": "raw",
          "encoding": "pcm_s16le",
          "sample_rate": 16000
        },
        "transcription_config": {
          "language": "en",
          "operating_point": "enhanced",
          "output_locale": "en-US",
          "additional_vocab": ["gnocchi", "bucatini", "bigoli"],
          "diarization": "speaker",
          "enable_partials": false
        },
        "translation_config": {
          "target_languages": [],
          "enable_partials": false
        },
        "audio_events_config": {
          "types": ["applause", "music"]
        }
      });
      
      tokio::spawn(async move {
        let mut last_seq_no = 0;
        while let Some(data) = receiver.next().await {
            if let Ok(data) = data {
                match data {
                    axum::extract::ws::Message::Text(utf8_bytes) => {
                        if utf8_bytes.as_str() == "START_VOICE_RECORDING" {
                            speechmatics_sender.send(tungstenite::Message::Text(msg.to_string())).await.unwrap();
                        }
                        if utf8_bytes.as_str() == "STOP_VOICE_RECORDING" {
                            let close_msg = serde_json::json!({
                                "message": "EndOfStream",
                                "last_seq_no": last_seq_no
                            });
                            speechmatics_sender.send(tungstenite::Message::Text(close_msg.to_string())).await.unwrap();
    
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
        let _guard = SpeechmaticsReceiverDrop;

        loop {
            let value = speechmatics_receiver.next().await;
            if let Some(value) = value {
                match value {
                    Ok(msg) => {
                        log::info!("{:?}", msg);
                    },
                    Err(err) => {
                        log::error!("{:?}", err);
                    },
                }
            }
        }
    });

}
