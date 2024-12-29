mod auth;
mod container;
mod database;
mod docker;
mod route;
mod secret;

use std::{net::Ipv4Addr, sync::Arc};

use crate::auth::cookie;
use askama_axum::IntoResponse;
use auth::{cookie::Key, AuthenticatedUser};
use axum::{extract::State, middleware::from_extractor_with_state, routing::get, Router};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use bollard::Docker;
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
    CookieDecodeError(base64::DecodeError),
    #[error("Bad cookie key length")]
    BadCookieKeyLength { expected: usize, actual: usize },
    #[error("Error reading database URL: {0}")]
    DatabaseUrlError(#[from] std::env::VarError),
    #[error("Bad database key encoding: {0}")]
    BadDatabaseKey(base64::DecodeError),
    #[error("Error initializing database: {0}")]
    DatabaseError(#[from] database::InitializeError),
}

async fn update(State(state): State<TugState>) -> impl IntoResponse {
    container::update(&state.docker, &state.update_lock).await
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
    connection: libsql::Connection,
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

    let url = std::env::var("LIBSQL_URL").map_err(TugError::DatabaseUrlError)?;
    let key = BASE64_URL_SAFE_NO_PAD
        .decode(secrets.database_encryption_key.as_ref())
        .map_err(TugError::BadDatabaseKey)?
        .into();

    let connection = database::initialize(url, secrets.lib_sql_auth_token.clone(), key).await?;

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
        connection,
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
        .merge(route::create_router())
        .route_layer(from_extractor_with_state::<AuthenticatedUser, _>(
            state.clone(),
        ))
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
