use async_std::path::PathBuf;
use futures::{SinkExt, StreamExt};
use tokio::fs::File;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use url::Url;
use base64::{engine::general_purpose, Engine as _};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::tungstenite::Message;
use tokio::io::AsyncReadExt;

#[derive(Serialize, Deserialize)]
struct StartRecognition {
    message: String,
    audio_format: AudioFormat,
}

#[derive(Serialize, Deserialize)]
struct AudioFormat {
    r#type: String,
    encoding: String,
    sample_rate: u32,
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let url = Url::parse("wss://eu2.rt.speechmatics.com/v2?jwt=ieDYfZVXcfmLpVKzLVQ64C9BbdJznb6O").unwrap();
    let sec_key: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();
    let b64 = general_purpose::STANDARD.encode(sec_key);
    println!("{}", b64);
    let request = http::Request::builder()
        .method("GET")
        .uri(url.as_str())
        .header("Host", "eu2.rt.speechmatics.com")
        .header("Authorization", "Bearer ieDYfZVXcfmLpVKzLVQ64C9BbdJznb6O")
        .header("Sec-WebSocket-Key", b64)
        .header("Connection", "keep-alive, Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .body(())
        .unwrap();

    let (ws_stream, ws_response) = connect_async(request).await.expect("Failed to connect");
    println!("{:?}", ws_response);
    println!("WebSocket handshake has been successfully completed");
    start_recognition(ws_stream).await;
}

async fn start_recognition(ws_stream: WebSocketStream<MaybeTlsStream<TcpStream>>) {

    let test_file_path = PathBuf::new()
        .join(".")
        .join("tests")
        .join("data")
        .join("example.wav");

    let mut file = File::open(test_file_path).await.unwrap();
    let mut data = Vec::new();
    file.read_to_end(&mut data).await.unwrap();
    let ( mut sender, mut receiver) = ws_stream.split();

    let msg = serde_json::json!({
        "message": "StartRecognition",
        "audio_format": {
          "type": "raw",
          "encoding": "pcm_f32le",
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

      let close_msg = serde_json::json!({
        "message": "EndOfStream",
        "last_seq_no": 1
      });
    sender.send(Message::Text(msg.to_string())).await.unwrap();
    sender.send(Message::binary(&data[..])).await.unwrap();
    sender.send(Message::Text(close_msg.to_string())).await.unwrap();

    loop {
        let value = &receiver.next().await;
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

}

// async fn send_audio_data(ws_stream: &mut tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>) {
//     let mut file = tokio::fs::File::open("audio.raw").await.unwrap();
//     let mut buffer = [0; 1024];

//     while let Ok(n) = file.read(&mut buffer).await {
//         if n == 0 {
//             break;
//         }
//         ws_stream.send(Message::Binary(buffer[..n].to_vec())).await.unwrap();
//     }
// }

// async fn handle_responses(ws_stream: &mut tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>) {
//     while let Some(msg) = ws_stream.next().await {
//         match msg {
//             Ok(Message::Text(text)) => {
//                 println!("Received: {}", text);
//                 // Parse and handle the JSON message as needed
//             }
//             Ok(Message::Binary(_)) => {
//                 // Handle binary messages if applicable
//             }
//             _ => {}
//         }
//     }
// }

// async fn close_connection(ws_stream: &mut tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>) {
//     ws_stream.close(None).await.unwrap();
// }