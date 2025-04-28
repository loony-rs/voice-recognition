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
use speechmatics::microsoft::{set_callbacks, speech_recognizer_from_push_stream, MsConfig};
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
        println!("Websocket closed.");
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
                println!("Received text: {}", text);
                if text == "START_VOICE_RECORDING" {
                    log::info!("Voice recording started.");
                }
                if text == "STOP_VOICE_RECORDING" {
                    log::info!("Voice recording stopped.");
                    push_stream.close_stream().unwrap();
                    break;
                }
               
            }
            AxumMessage::Binary(bin) => {
                println!("Received bytes");
                push_stream.write(bin).unwrap();
            }
            _ => {}
        }
    }
    log::info!("socket work finished");
    handle.await.unwrap();
    log::info!("Finished.");
    socket.close().await.unwrap();

}

struct AppState {
    config: MsConfig
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let ms_subscription_key = env::var("MSSubscriptionKey").unwrap();
    let ms_service_region = env::var("MSServiceRegion").unwrap();

    let app_state = AppState {
        config: MsConfig {  ms_service_region, ms_subscription_key }
    };
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .with_state(Arc::new(app_state));
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:2000").await.unwrap();

    log::info!("Listening on port 2000");
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
