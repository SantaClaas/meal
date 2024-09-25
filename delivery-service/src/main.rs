use std::{collections::HashMap, sync::Arc};

use axum::{
    body::Bytes,
    extract::{
        ws::{self, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    http::{HeaderValue, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use tokio::sync::{mpsc, Mutex};
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod extractor;

#[derive(Clone, Default)]
struct AppState {
    channels: Arc<Mutex<HashMap<Arc<str>, mpsc::Sender<Bytes>>>>,
}

#[tokio::main]
async fn main() {
    // To debug axum extractor rejections see https://docs.rs/axum/latest/axum/extract/index.html#logging-rejections
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting");

    let mut app = Router::new()
        .route("/messages/:to", post(create_message))
        .route("/messages/:to", get(subscribe_messages));

    // Not looking nice, but functional to have CORS only in development
    #[cfg(debug_assertions)]
    {
        app = app.layer(
            CorsLayer::new()
                .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET]),
        );
    }

    let app = app.with_state(Default::default());

    // let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

/// This endpoint receives messages sent by clients to be delivered to other clients
async fn create_message(
    State(state): State<AppState>,
    Path(to): Path<Arc<str>>,
    bytes: Bytes,
) -> impl IntoResponse {
    let channels = state.channels.lock().await;
    //TODO think about not leaking if they exist or not
    //TODO think about leaking data through timings
    let Some(sender) = channels.get(to.as_ref()) else {
        return StatusCode::NOT_FOUND;
    };

    if let Err(error) = sender.send(bytes).await {
        tracing::error!("Error sending message {:?}", error);
    }

    StatusCode::CREATED
}

async fn handle_socket(mut socket: WebSocket, mut receiver: mpsc::Receiver<Bytes>) {
    // We only use the socket unidirectional for now
    while let Some(message) = receiver.recv().await {
        let message = ws::Message::Binary(message.into());
        if let Err(result) = socket.send(message).await {
            tracing::error!("Error sending message through websocket: {}", result)
        }
    }

    tracing::debug!("Message stream closed")
}

/// Handles requests for listening to server sent events (SSE) that are used to send incoming messages from the server to the client
async fn subscribe_messages(
    State(state): State<AppState>,
    websocket: WebSocketUpgrade,
    Path(client_id): Path<Arc<str>>,
) -> impl IntoResponse {
    //TODO add cryptographic client authentication to avoid session hijacking
    //TODO (continued) (even though the hijacker can't read the messages most likely, they can analyze traffic meta data for that client)

    //TODO decide on buffer size
    let (sender, receiver) = mpsc::channel(8);
    // Register sender for this id
    let previous_sender = state.channels.lock().await.insert(client_id, sender);
    if let Some(previous_sender) = previous_sender {
        //TODO think about if this is valid
        tracing::warn!("Replacing previous subscriber for client");
        // This should close the SSE stream for the other client that used the same id
        //TODO test assumption
        drop(previous_sender);
    }

    //TODO keep alive
    websocket.on_upgrade(|socket| handle_socket(socket, receiver))
}
