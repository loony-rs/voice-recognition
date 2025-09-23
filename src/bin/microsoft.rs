use std::{sync::Arc, time::Duration};

use axum::{
    extract::WebSocketUpgrade,
    response::IntoResponse,
    routing::get,
    Router,
    extract::State
};
use futures::{stream::StreamExt, SinkExt};
use axum::extract::ws::{Message as AxumMessage, WebSocket};
use voice_recognition::microsoft::{set_callbacks, speech_recognizer_from_push_stream, MsConfig};
use tokio::time::sleep;
use std::env;

// WebSocket handler
async fn ws_handler(ws: WebSocketUpgrade, State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let config  = MsConfig {
        ms_subscription_key: state.config.ms_subscription_key.clone(),
        ms_service_region: state.config.ms_service_region.clone(),
    };
    ws.on_upgrade(|socket: WebSocket| async {
        handle_socket(socket, config).await;
    })
}

// Function that handles the actual websocket connection
async fn handle_socket(mut socket: WebSocket, config: MsConfig) {
    println!("WebSocket connection established");
    let (mut speech_recognizer, mut push_stream ) = speech_recognizer_from_push_stream(config);

    let handle = tokio::spawn(async move {
       set_callbacks(&mut speech_recognizer);

        if let Err(err) = speech_recognizer.start_continuous_recognition_async().await {
            println!("start_continuous_recognition_async error {:?}", err);
        }

        sleep(Duration::from_secs(10)).await;
    });
    
    while let Some(Ok(msg)) = socket.next().await {
        match msg {
            AxumMessage::Text(text) => {
                if text == "START_VOICE_RECORDING" {
                    log::debug!("START_VOICE_RECORDING");
                }
                if text == "STOP_VOICE_RECORDING" {
                    log::debug!("STOP_VOICE_RECORDING");
                    push_stream.close_stream().unwrap();
                    break;
                }
               
            }
            AxumMessage::Binary(bin) => {
                push_stream.write(bin).unwrap();
            }
            _ => {}
        }
    }
    handle.await.unwrap();
    socket.close().await.unwrap();
    println!("Websocket closed.");
}

struct AppState {
    config: MsConfig
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let ms_subscription_key = env::var("MSSubscriptionKey").unwrap();
    let ms_service_region = env::var("MSServiceRegion").unwrap();
    let port = std::env::var("PORT").unwrap_or("2000".to_string());
    let port = port.parse::<i32>().unwrap();

    let app_state = AppState {
        config: MsConfig {  ms_service_region, ms_subscription_key }
    };
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(Arc::new(app_state));
    let url = format!("127.0.0.1:{}", port);
    
    let listener = tokio::net::TcpListener::bind(&url).await.unwrap();

    log::info!("Listening on {}", url);

    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
