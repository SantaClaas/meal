use std::{collections::HashMap, sync::Arc};

use axum::{
    body::Bytes,
    extract::{
        ws::{self, WebSocket},
        Path, State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use tokio::sync::{mpsc, Mutex};
use tower_http::{
    services::{ServeDir, ServeFile},
    set_status::SetStatus,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod extractor;

#[derive(Clone, Default)]
struct AppState {
    channels: Arc<Mutex<HashMap<Arc<str>, mpsc::Sender<Bytes>>>>,
}

/// The single page application setup used in production. During development a vite proxy is used to host the app and
/// proxy delivery service requests for a better development experience.
fn serve_client() -> ServeDir<SetStatus<ServeFile>> {
    ServeDir::new("./app")
        // If the route is a client side navigation route, this will serve the app and let the app router take over the
        // path handling after the app is loaded
        .not_found_service(ServeFile::new("./app/index.html"))
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

    let app = Router::new()
        .route("/messages/:to", post(create_message))
        .route("/messages/:to", get(subscribe_messages))
        .fallback_service(serve_client())
        .with_state(Default::default());

    // It might become annoying to get the mac pop up for running on 0.0.0.0 so change to localhost instead
    let listener = tokio::net::TcpListener::bind(if cfg!(debug_assertions) {
        "127.0.0.1:3000"
    } else {
        "0.0.0.0:3000"
    })
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

async fn handle_socket(mut socket: WebSocket, State(state): State<AppState>, client_id: Arc<str>) {
    //TODO keep alive
    //TODO add client authentication to avoid session hijacking
    // Hijackers can deny messages to the client and analyze meta data but not read messages if they don't have the clients credentials

    //TODO decide on buffer size
    let (sender, mut receiver) = mpsc::channel(8);

    // Register sender for this id
    // Immediately drop lock after insert to avoid deadlock
    let previous_sender = state
        .channels
        .lock()
        .await
        .insert(client_id.clone(), sender);

    if let Some(previous_sender) = previous_sender {
        //TODO think about if this is valid
        tracing::warn!("Replacing previous subscriber for client");
        // This should close the SSE stream for the other client that used the same id
        //TODO test assumption
        drop(previous_sender);
    }

    tracing::debug!("Websocket established");
    loop {
        tokio::select! {
            Some(message) = receiver.recv() => {
                let message = ws::Message::Binary(message.into());
                if let Err(error) = socket.send(message).await {
                    tracing::error!("Error sending message through websocket: {}", error);
                    //TODO remove channel to avoid memory leak. It is the clients responsibility to reestablish a connection
                    break;
                }
            },
            // We only use the socket unidirectional for now
            // but we want to know when the client closes the socket
            Some(Ok(_)) = socket.recv() => {},
            else => break,
        }
    }

    tracing::debug!("Message stream closed");

    // Try to close gracefully but if not ignore error
    if let Err(error) = socket.close().await {
        tracing::trace!("Ignoring error closing websocket: {}", error);
    }

    // Remove channel to avoid memory leak
    // It is the clients responsibility to reestablish a new connection
    state.channels.lock().await.remove(&client_id);
}

/// Handles requests for listening to server sent events (SSE) that are used to send incoming messages from the server to the client
async fn subscribe_messages(
    state: State<AppState>,
    websocket: WebSocketUpgrade,
    Path(client_id): Path<Arc<str>>,
) -> impl IntoResponse {
    websocket.on_upgrade(|socket| handle_socket(socket, state, client_id))
}
