mod auth;
mod container;
mod docker;
mod secret;

use std::{collections::HashMap, env::VarError, net::Ipv4Addr, sync::Arc};

use crate::auth::cookie;
use askama::Template;
use askama_axum::IntoResponse;
use axum::{extract::State, routing::get, Router};
use bollard::Docker;
use container::UpdateResult;
use secret::Secrets;
use tokio::{signal, sync::Mutex};
use tower_http::validate_request::ValidateRequestHeaderLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(thiserror::Error, Debug)]
enum TugError {
    #[error("Error setting up secrets")]
    SecretError(#[from] secret::Error),
    #[error("Error setting up docker")]
    DockerError(#[from] bollard::errors::Error),
    #[error("Error getting cookie key")]
    CookieKeyError,
}

async fn update(State(state): State<TugState>) -> impl IntoResponse {
    container::update(&state.docker, &state.update_lock).await
}

#[derive(Template, Default)]
#[template(path = "index.html")]
struct IndexTemplate;

async fn get_index(State(_state): State<TugState>) -> IndexTemplate {
    // create
    IndexTemplate
}

async fn shutdown_signal() {
    let control_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = control_c => {},
        _ = terminate => {},
    }

    tracing::info!("Termination requested. Shutting down");
}

#[derive(Clone)]
struct TugState {
    docker: Docker,
    secrets: Secrets,
    cookie_key: cookie::Key,
    // Is there a better primitive to have one task exlusively running the update
    /// Lock to avoid multiple updates at the same time
    /// Does not lock the docker instance as other tasks are still permitted
    update_lock: Arc<Mutex<()>>,
}

#[tokio::main]
async fn main() -> Result<(), TugError> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=trace,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    #[cfg(debug_assertions)]
    dotenvy::dotenv().expect("Expected to load .env file in development");

    let secrets = secret::setup().await?;
    tracing::info!("Setting up docker");
    let docker = docker::set_up()?;
    let update_lock: Arc<Mutex<()>> = Arc::default();
    let state = TugState {
        secrets,
        cookie_key: cookie::Key::new().ok_or(TugError::CookieKeyError)?,
        docker: docker.clone(),
        update_lock: update_lock.clone(),
    };

    tokio::spawn(async move {
        tracing::info!("Running initial update");
        match container::update(&docker, update_lock.as_ref()).await {
            Ok(UpdateResult::Completed) => tracing::info!("Initial update completed"),
            Ok(UpdateResult::AlreadyStarted) => tracing::info!("Initial update already started"),
            Err(error) => tracing::error!("Error running initial update: {error}"),
        }
    });

    let app = Router::new()
        .route("/update", get(update))
        .route("/", get(get_index))
        .route("/signin", get(auth::get_sign_in).post(auth::create_sign_in))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind((Ipv4Addr::new(0, 0, 0, 0), 3001))
        .await
        .unwrap();

    tracing::info!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
    Ok(())
}
