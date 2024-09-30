use std::{collections::HashMap, env::VarError, net::Ipv4Addr};

use askama::Template;
use axum::{extract::State, http::StatusCode, routing::get, Router};
use bollard::{
    container::{
        self, CreateContainerOptions, InspectContainerOptions, ListContainersOptions,
        RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
    },
    image::CreateImageOptions,
    secret::{ContainerInspectResponse, ContainerState},
    Docker, API_DEFAULT_VERSION,
};
use tokio_stream::StreamExt;
use tower_http::validate_request::ValidateRequestHeaderLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(thiserror::Error, Debug)]
enum TugError {
    #[error("Error reading togboat token: {0}")]
    ReadTokenError(#[from] VarError),
    #[error(transparent)]
    DockerError(#[from] bollard::errors::Error),
    #[error("Expected newly created image to have an id")]
    NoImageId,
    #[error("Expected container to have an id or name")]
    NoContainerId,
}

async fn update_container(docker: &Docker) -> Result<(), TugError> {
    // 1. Check if container already exists
    // 2. Create image
    //    This pulls the latest image
    // 3. Create container
    //    Delete the old one if it exists
    // 4. Start container

    const CONTAINER_NAME: &str = "tugged-melt";
    const IMAGE_NAME: &str = "ghcr.io/santaclaas/meal:main";

    // Check if containers already exists

    let result = docker
        .inspect_container(CONTAINER_NAME, None::<InspectContainerOptions>)
        .await;

    // 404 Not found is okay
    let container = result.map(Option::Some).or_else(|orrer| match orrer {
        bollard::errors::Error::DockerResponseServerError {
            status_code: 404,
            message: _,
        } => Ok(None),
        other => Err(other),
    })?;

    // If the image is none, then it is invalid and needs to be updated
    let container_image_id = if let Some(container_image) = container
        .as_ref()
        .and_then(|container| container.image.clone())
    {
        let image = docker.inspect_image(&container_image).await?;
        // Just to see if they are the same
        if image.id.clone().is_some_and(|id| id == container_image) {
            tracing::debug!("They are the same!");
        }

        image.id
    } else {
        None
    };

    tracing::debug!("Container image id: {:?}", container_image_id);

    // Pull latest image
    let options = Some(CreateImageOptions {
        // Allways include the tag in the name
        from_image: IMAGE_NAME,
        platform: "linux/amd64",
        ..Default::default()
    });

    tracing::debug!("Pulling image");

    let mut responses = docker.create_image(options, None, None);
    while let Some(result) = responses.next().await {
        let information = result?;
        tracing::debug!("Create image: {:?}", information.status);
    }

    // Get newly pulled image
    let image = docker.inspect_image(IMAGE_NAME).await?;
    tracing::debug!("New image id: {:?}", image.id);
    let new_id = image.id.ok_or(TugError::NoImageId)?;

    if container_image_id.is_some_and(|id| id == new_id) {
        tracing::debug!("Container is up to date");
        return Ok(());
    }

    // Stop container if it exists
    if let Some(container) = container {
        let id = container
            .id
            .or(container.name)
            .ok_or(TugError::NoContainerId)?;

        tracing::debug!("Stopping container");
        // Returns 304 if the container is not running
        docker
            .stop_container(&id, None::<StopContainerOptions>)
            .await?;

        docker
            .remove_container(&id, None::<RemoveContainerOptions>)
            .await?;
    }

    // Create container
    let options = Some(CreateContainerOptions {
        name: CONTAINER_NAME,
        platform: Some("linux/amd64"),
    });

    let configuration = container::Config {
        image: Some(IMAGE_NAME),
        ..Default::default()
    };
    tracing::debug!("Creating container",);

    let response = docker.create_container(options, configuration).await?;

    tracing::debug!("Starting container");
    docker
        .start_container(&response.id, None::<StartContainerOptions<String>>)
        .await?;

    tracing::debug!("Started container");

    Ok(())
}

fn set_up_docker() -> Result<Docker, bollard::errors::Error> {
    if cfg!(debug_assertions) {
        // I think this is docker desktop specific
        Docker::connect_with_socket(
            "/Users/claas/.docker/run/docker.sock",
            120,
            API_DEFAULT_VERSION,
        )
        // Docker::connect_with_unix("unix:///var/run/docker.sock", 120, API_DEFAULT_VERSION)
        // Docker::connect_with_unix_defaults()
    } else {
        Docker::connect_with_defaults()
    }
}

#[derive(Template, Default)]
#[template(path = "index.html")]
struct IndexTemplate;

async fn get_index(State(_state): State<AppState>) -> IndexTemplate {
    // create
    IndexTemplate
}

async fn update(State(state): State<AppState>) -> StatusCode {
    match update_container(&state.docker).await {
        Ok(()) => StatusCode::OK,
        Err(error) => {
            tracing::error!("Error starting container: {error}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

#[derive(Clone)]
struct AppState {
    docker: Docker,
}

#[tokio::main]
async fn main() -> Result<(), TugError> {
    #[cfg(debug_assertions)]
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=trace,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    std::env::set_var("TUGBOAT_TOKEN", "test");

    let token = std::env::var("TUGBOAT_TOKEN")?;
    let docker = set_up_docker()?;
    let state = AppState { docker };
    let app = Router::new()
        .route("/update", get(update))
        .layer(ValidateRequestHeaderLayer::bearer(&token))
        .route("/", get(get_index))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind((Ipv4Addr::new(127, 0, 0, 1), 3001))
        .await
        .unwrap();

    tracing::debug!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
    Ok(())
}
