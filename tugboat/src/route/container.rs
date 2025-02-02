use std::{collections::HashMap, sync::Arc, time::Duration};

use askama::Template;
use axum::{
    extract::{Path, State},
    http::{self, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Form,
};
use bollard::{
    container::{
        self, CreateContainerOptions, InspectContainerOptions, ListContainersOptions,
        RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
    },
    image::CreateImageOptions,
    secret::{HostConfig, PortBinding},
    Docker,
};
use environment::try_load_from_file;
use libsql::named_params;
use serde::Deserialize;
use sha2::Digest;
use tokio_stream::StreamExt;

use crate::{route::token::Token, TugState};

use super::token;

struct Container {
    id: Arc<str>,
    name: Arc<str>,
    image: Arc<str>,
}

#[derive(Template)]
#[template(path = "container/index.html")]
pub(super) struct IndexTemplate {
    containers: Vec<Container>,
    image_suggestions: Option<Vec<Arc<str>>>,
}

#[derive(thiserror::Error, Debug)]
enum CreateIndexTemplateError {
    #[error("Error listing containers: {0}")]
    GetContainersError(#[from] GetContainersError),
}

impl IndexTemplate {
    async fn from(docker: &Docker) -> Result<Self, CreateIndexTemplateError> {
        let (containers_result, images_result) =
            tokio::join!(get_containers(docker), get_images(docker));

        Ok(IndexTemplate {
            containers: containers_result?,
            image_suggestions: images_result
                .inspect_err(|error| tracing::error!("Error getting images: {error}"))
                .ok(),
        })
    }
}

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub(super) struct GetIndexError(#[from] CreateIndexTemplateError);

impl IntoResponse for GetIndexError {
    fn into_response(self) -> Response {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

#[derive(thiserror::Error, Debug)]
pub(super) enum GetContainersError {
    #[error("Error listing containers: {0}")]
    ListError(bollard::errors::Error),
    #[error("Container with no id")]
    NoId,
    #[error("Container with no name")]
    NoName,
    #[error("Container with no image")]
    NoImage,
}

async fn get_containers(docker: &Docker) -> Result<Vec<Container>, GetContainersError> {
    docker
        .list_containers(Some(ListContainersOptions {
            filters: HashMap::from([("label", vec![label::TAG])]),
            ..Default::default()
        }))
        .await
        .map_err(GetContainersError::ListError)?
        .into_iter()
        .map(|container| -> Result<Container, GetContainersError> {
            let id = container.id.ok_or(GetContainersError::NoId)?;

            let name = container
                .names
                .as_ref()
                .and_then(|names| names.first())
                .ok_or(GetContainersError::NoName)?
                .trim_start_matches('/');

            let image = container.image.ok_or(GetContainersError::NoImage)?;

            Ok(Container {
                id: id.as_str().into(),
                name: name.into(),
                image: image.into(),
            })
        })
        .collect()
}

async fn get_images(docker: &Docker) -> Result<Vec<Arc<str>>, bollard::errors::Error> {
    let images = docker.list_images::<&str>(None).await?;
    let images = images
        .into_iter()
        .flat_map(|image| image.repo_tags)
        .map(Arc::from)
        .collect();
    Ok(images)
}

pub(super) async fn get_index(
    State(state): State<TugState>,
) -> Result<IndexTemplate, GetIndexError> {
    IndexTemplate::from(&state.docker)
        .await
        .map_err(GetIndexError)
}

#[derive(Deserialize)]
struct Port(u16);

#[derive(Deserialize)]
pub(super) struct CreateRequest {
    name: Arc<str>,
    image: Arc<str>,
    container_port: Option<Port>,
    host_port: Option<Port>,
    environment_file: Option<Arc<str>>,
}

#[derive(thiserror::Error, Debug)]
pub(super) enum CreateError {
    #[error("Error creating image: {0}")]
    CreateImageError(bollard::errors::Error),
    #[error("Error creating container: {0}")]
    CreateContainerError(bollard::errors::Error),
    #[error("Error starting container: {0}")]
    StartContainerError(bollard::errors::Error),
    #[error("Error getting index template: {0}")]
    GetIndexError(#[from] CreateIndexTemplateError),
}

impl IntoResponse for CreateError {
    fn into_response(self) -> Response {
        tracing::error!("Error creating container: {:?}", self);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

mod label {
    /// Tag to mark containers managed by tugboat
    pub(super) const TAG: &str = "moe.cla.tugboat.tugged";
}

mod environment {

    pub(super) fn try_load_from_file(path: impl AsRef<str>) -> Option<Vec<String>> {
        let variables = dotenvy::from_path_iter(path.as_ref())
            .inspect_err(|error| {
                tracing::error!("Error loading environment variables from file: {error}")
            })
            .ok()?
            .filter_map(|item| match item {
                Ok((key, value)) => Some(format!("{key}={value}")),
                Err(error) => {
                    tracing::error!("Error loading environment variable: {error}");
                    None
                }
            })
            .collect();

        Some(variables)
    }
}

pub(super) async fn create(
    State(state): State<TugState>,
    Form(request): Form<CreateRequest>,
) -> Result<Redirect, CreateError> {
    // Pull latest image
    tracing::debug!("Pulling image");
    let options = Some(CreateImageOptions {
        // Always include the tag in the name
        from_image: request.image.as_ref(),
        platform: "linux/amd64",
        ..Default::default()
    });

    let mut responses = state.docker.create_image(options, None, None);
    while let Some(result) = responses.next().await {
        let information = result.map_err(CreateError::CreateImageError)?;
        tracing::debug!("Create image: {:?}", information.status);
    }

    tracing::debug!("Creating container");
    let options = Some(CreateContainerOptions {
        name: request.name.as_ref(),
        platform: Some("linux/amd64"),
    });

    let host_configuration = if let (Some(container_port), Some(host_port)) =
        (request.container_port, request.host_port)
    {
        Some(HostConfig {
            port_bindings: Some(HashMap::from([(
                // "3000/tcp".to_string(),
                format!("{}/tcp", container_port.0),
                Some(vec![PortBinding {
                    host_ip: Some("0.0.0.0".to_string()),
                    host_port: Some(host_port.0.to_string()),
                }]),
            )])),
            ..Default::default()
        })
    } else {
        None
    };

    // Load environment variables from file
    let variables = request.environment_file.and_then(try_load_from_file);

    let configuration = container::Config::<String> {
        image: Some(String::from(request.image.as_ref())),
        // exposed_ports: Some(HashMap::from([("3000", HashMap::default())])),
        host_config: host_configuration,
        labels: Some(HashMap::from([(label::TAG.to_owned(), String::default())])),
        env: variables,
        ..Default::default()
    };

    let response = state
        .docker
        .create_container(options, configuration)
        .await
        .map_err(CreateError::CreateContainerError)?;

    tracing::debug!("Starting container");
    state
        .docker
        .start_container::<&str>(&response.id, None)
        .await
        .map_err(CreateError::StartContainerError)?;

    tracing::debug!("Started container");

    Ok(Redirect::to("/containers"))
}

#[derive(thiserror::Error, Debug)]
pub(super) enum CreateTokenError {
    #[error("Error loading container: {0}")]
    LoadContainerError(bollard::errors::Error),
    #[error("Error creating token: {0}")]
    CreateTokenError(#[from] token::CreateError),
    #[error("Error hashing token: {0}")]
    HashTokenError(#[from] token::HashError),
    #[error("Error preparing SQL statement: {0}")]
    PrepareStatementError(libsql::Error),
    #[error("Error executing SQL statement: {0}")]
    ExecuteStatementError(libsql::Error),
}

impl IntoResponse for CreateTokenError {
    fn into_response(self) -> axum::response::Response {
        if let Self::LoadContainerError(bollard::errors::Error::DockerResponseServerError {
            status_code: 404,
            message: _,
        }) = self
        {
            return StatusCode::NOT_FOUND.into_response();
        }

        axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

#[derive(Template)]
#[template(path = "container/token.html")]
pub(super) struct CreateTokenResultTemplate {
    token: Arc<str>,
}

pub(super) async fn create_token(
    State(state): State<TugState>,
    Path(container_id): Path<Arc<str>>,
) -> Result<CreateTokenResultTemplate, CreateTokenError> {
    let _container = state
        .docker
        .inspect_container(container_id.as_ref(), None)
        .await
        .map_err(CreateTokenError::LoadContainerError)?;

    let token = Token::new()?;

    let hash = token.hash()?;

    // Store hash in database
    let mut statement = state
        .connection
        .prepare(
            "INSERT OR REPLACE INTO tokens (container_id, token_hash) VALUES (:container_id, :token_hash)",
        )
        .await
        .map_err(CreateTokenError::PrepareStatementError)?;

    let updated_rows = statement
        .execute(named_params! {
            ":container_id": container_id.as_ref(),
            ":token_hash": hash.as_ref(),
        })
        .await
        .map_err(CreateTokenError::ExecuteStatementError)?;

    assert_eq!(updated_rows, 1, "Expected to update one row");

    Ok(CreateTokenResultTemplate {
        token: token.to_base64(),
    })
}

/// Runs forever and cleans up expired app data
pub(crate) async fn collect_garbage(state: TugState) {
    // It is not important that it cleans exactly, but it is important that it happens regularly
    // Duration from minutes is experimental currently
    let mut interval = tokio::time::interval(Duration::from_secs(24 * 60 * 60));
    loop {
        interval.tick().await;
        // Clean up dead containers
        let result = state
            .docker
            .list_containers(Some(ListContainersOptions {
                all: true,
                filters: HashMap::from([("label", vec![label::TAG])]),
                ..Default::default()
            }))
            .await;

        let container_ids: Vec<String> = match result {
            Ok(containers) => containers
                .into_iter()
                .filter_map(|container| container.id)
                .collect(),
            Err(error) => {
                tracing::error!("Error listing containers for database cleanup: {:?}", error);
                continue;
            }
        };

        if container_ids.is_empty() {
            continue;
        }

        if container_ids.len() >= i16::MAX as usize {
            tracing::error!(
                "Expected containers to not exceed parameter limit, got {}",
                container_ids.len()
            );
            continue;
        }

        let query = format!(
            "DELETE FROM tokens WHERE container_id NOT IN ({})",
            (1..=container_ids.len())
                .map(|index| format!("?{index}"))
                .collect::<Vec<_>>()
                .join(", ")
        );

        match state.connection.execute(&query, container_ids).await {
            Ok(deleted_rows) => {
                tracing::info!(
                    "Cleaned up {deleted_rows} containers from database that no longer exist"
                )
            }
            Err(error) => {
                tracing::error!("Error cleaning up containers from database: {:?}", error)
            }
        };
    }
}

pub(super) enum UpdateResult {
    Completed,
    AlreadyStarted,
}

impl IntoResponse for UpdateResult {
    fn into_response(self) -> axum::response::Response {
        match self {
            UpdateResult::Completed => StatusCode::OK.into_response(),
            UpdateResult::AlreadyStarted => StatusCode::ACCEPTED.into_response(),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub(super) enum UpdateError {
    #[error(transparent)]
    DockerError(#[from] bollard::errors::Error),
    #[error("Expected newly created image to have an id")]
    NoImageId,
    #[error("Container was configured without image")]
    NoImage,
    #[error("Container has no name")]
    NoContainerName,
}

impl IntoResponse for UpdateError {
    fn into_response(self) -> axum::response::Response {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

/// This handler expects that the request has been authorized and the authorization is valid.
/// Authorization is not part of this handlers responsibilities.
/// This means the container exists at least in our database.
pub(super) async fn update(
    State(state): State<TugState>,
    Path(container_id): Path<Arc<str>>,
) -> Result<UpdateResult, UpdateError> {
    //TODO remove clones
    //TODO run update in background and immediately respond with Accepted (202) status code
    let mut locks = state.update_locks.lock().await;
    let update_lock = locks.entry(container_id.clone()).or_default();

    // Don't interfere with running updates. Could make things awkward
    // Technically the current update should be cancelled as it might have pulled the last image which is now outdated
    // but we don't expect so many updates to happen at the same time for this to become a problem
    let Ok(_lock) = update_lock.try_lock() else {
        tracing::debug!("Update already underway");
        return Ok(UpdateResult::AlreadyStarted);
    };

    // Check if containers already exists
    let result = state
        .docker
        .inspect_container(container_id.as_ref(), None::<InspectContainerOptions>)
        .await;

    // 404 Not found is okay
    /// This constant can be inlined in future Rust versions
    const NOT_FOUND: u16 = http::StatusCode::NOT_FOUND.as_u16();
    let container = result.map(Option::Some).or_else(|error| match error {
        bollard::errors::Error::DockerResponseServerError {
            status_code: NOT_FOUND,
            message: _,
        } => Ok(None),
        other => Err(other),
    })?;

    // If the image id is none, then it is invalid and needs to be updated
    let container_image_id = if let Some(container_image) = container
        .as_ref()
        .and_then(|container| container.image.clone())
    {
        let image = state.docker.inspect_image(&container_image).await?;
        // Just to see if they are the same
        if image.id.clone().is_some_and(|id| id == container_image) {
            tracing::debug!("They are the same!");
        }

        image.id
    } else {
        None
    };

    let image_name = container
        .clone()
        .and_then(|container| container.config)
        .and_then(|configuration| configuration.image)
        .ok_or(UpdateError::NoImage)?;

    tracing::debug!("Container image id: {:?}", container_image_id);

    // Pull latest image
    let options = Some(CreateImageOptions {
        // Always include the tag in the name
        from_image: image_name.clone(),
        platform: "linux/amd64".to_string(),
        ..Default::default()
    });

    tracing::debug!("Pulling image");

    let mut responses = state.docker.create_image(options, None, None);
    while let Some(result) = responses.next().await {
        let information = result?;
        tracing::debug!("Create image: {:?}", information.status);
    }

    // Get newly pulled image
    let image = state.docker.inspect_image(image_name.as_ref()).await?;
    tracing::debug!("New image id: {:?}", image.id);
    let new_id = image.id.ok_or(UpdateError::NoImageId)?;

    if container_image_id.is_some_and(|id| id == new_id) {
        tracing::debug!("Container is up to date");
        return Ok(UpdateResult::Completed);
    }

    let container_name = container
        .clone()
        .and_then(|container| container.name)
        .ok_or(UpdateError::NoContainerName)?;

    // Stop container if it exists
    if let Some(container) = container.clone() {
        let id = container.id.unwrap_or(container_name.clone());

        tracing::debug!("Stopping container");
        // Returns 304 if the container is not running
        state
            .docker
            .stop_container(&id, None::<StopContainerOptions>)
            .await?;

        state
            .docker
            .remove_container(&id, None::<RemoveContainerOptions>)
            .await?;
    }

    // Create container
    let options = Some(CreateContainerOptions {
        name: container_name.as_ref(),
        platform: Some("linux/amd64"),
    });

    let host_configuration = HostConfig {
        port_bindings: Some(HashMap::from([(
            "3000/tcp".to_string(),
            Some(vec![PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some("3000".to_string()),
            }]),
        )])),
        ..Default::default()
    };

    // Run with same environment variables
    let environment_variables = container
        .and_then(|container| container.config.and_then(|configuration| configuration.env));

    let configuration = container::Config {
        image: Some(image_name),
        // exposed_ports: Some(HashMap::from([("3000", HashMap::default())])),
        host_config: Some(host_configuration),
        env: environment_variables,
        ..Default::default()
    };

    tracing::debug!("Creating container",);

    let response = state
        .docker
        .create_container(options, configuration)
        .await?;

    tracing::debug!("Starting container");
    state
        .docker
        .start_container(&response.id, None::<StartContainerOptions<String>>)
        .await?;

    tracing::debug!("Started container");
    Ok(UpdateResult::Completed)
}
