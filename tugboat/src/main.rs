mod auth;
mod container;
mod docker;
mod secret;

use std::{net::Ipv4Addr, sync::Arc};

use crate::auth::cookie;
use askama::Template;
use askama_axum::IntoResponse;
use auth::{cookie::Key, AuthenticatedUser};
use axum::{extract::State, response::Redirect, routing::get, Router};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use bollard::{container::ListContainersOptions, secret::ContainerSummary, Docker};
use container::UpdateResult;
use secret::Secrets;
use tokio::{signal, sync::Mutex};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(thiserror::Error, Debug)]
enum TugError {
    #[error("Error setting up secrets")]
    SecretError(#[from] secret::Error),
    #[error("Error setting up docker")]
    DockerError(#[from] bollard::errors::Error),
    #[error("Error decoding cookie key")]
    CookieDecodeError(#[from] base64::DecodeError),
    #[error("Bad cookie key length")]
    BadCookieKeyLength { expected: usize, actual: usize },
}

async fn update(State(state): State<TugState>) -> impl IntoResponse {
    container::update(&state.docker, &state.update_lock).await
}

#[derive(Template, Default)]
#[template(path = "index.html")]
struct IndexTemplate {
    containers: Vec<ContainerSummary>,
}

#[derive(thiserror::Error, Debug)]
enum GetIndexError {
    #[error("Error getting containers: {0}")]
    DockerError(#[from] bollard::errors::Error),
}

impl IntoResponse for GetIndexError {
    fn into_response(self) -> axum::response::Response {
        axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

async fn get_index(
    State(state): State<TugState>,
    user: Option<AuthenticatedUser>,
) -> Result<impl IntoResponse, GetIndexError> {
    // Don't show container server information to unauthenticated users
    if user.is_none() {
        return Ok(Redirect::to("/signin").into_response());
    }

    let containers = state
        .docker
        .list_containers(Option::<ListContainersOptions<String>>::None)
        .await?;

    Ok(IndexTemplate { containers }.into_response())
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
    let cookie_key: [u8; Key::LENGTH] = BASE64_URL_SAFE_NO_PAD
        .decode(secrets.cookie_signing_secret.as_ref())
        .map_err(TugError::CookieDecodeError)?
        .try_into()
        .map_err(|secret: Vec<u8>| TugError::BadCookieKeyLength {
            expected: Key::LENGTH,
            actual: secret.len(),
        })?;

    let cookie_key = cookie::Key::from(cookie_key);

    tracing::info!("Setting up docker");
    let docker = docker::set_up()?;
    let update_lock: Arc<Mutex<()>> = Arc::default();
    let state = TugState {
        secrets,
        cookie_key,
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

    let address = if cfg!(debug_assertions) {
        Ipv4Addr::LOCALHOST
    } else {
        Ipv4Addr::new(0, 0, 0, 0)
    };

    let listener = tokio::net::TcpListener::bind((address, 3001))
        .await
        .unwrap();

    tracing::info!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
    Ok(())
}
