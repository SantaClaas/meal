pub(in crate::route) mod environment;

use core::fmt;
use std::{collections::HashMap, sync::Arc, time::Duration};

use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use bollard::{
    container::{
        self, CreateContainerOptions, InspectContainerOptions, ListContainersOptions,
        RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
    },
    image::CreateImageOptions,
    secret::{ContainerInspectResponse, HostConfig, PortBinding},
    Docker,
};
use libsql::named_params;
use serde::Deserialize;
use tokio_stream::StreamExt;

use crate::{route::token::Token, TugState};

use super::token;

enum Status {
    Created,
    Running,
    Paused,
    Restarting,
    Removing,
    Exited,
    Dead,
    Unknown(String),
}

impl Status {
    fn is_running(&self) -> bool {
        matches!(self, Status::Running | Status::Restarting)
    }
}

impl fmt::Display for Status {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let status = match self {
            Self::Created => "created",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Restarting => "restarting",
            Self::Removing => "removing",
            Self::Exited => "exited",
            Self::Dead => "dead",
            Self::Unknown(status) => status,
        };
        write!(formatter, "{status}")
    }
}

impl From<String> for Status {
    fn from(status: String) -> Self {
        match status.as_ref() {
            "created" => Self::Created,
            "running" => Self::Running,
            "paused" => Self::Paused,
            "restarting" => Self::Restarting,
            "removing" => Self::Removing,
            "exited" => Self::Exited,
            "dead" => Self::Dead,
            other => Self::Unknown(other.to_owned()),
        }
    }
}
struct Container {
    id: Arc<str>,
    name: Arc<str>,
    image: Arc<str>,
    status: Option<Status>,
}

#[derive(Template)]
#[template(path = "container/index.html")]
pub(super) struct IndexTemplate {
    containers: Vec<Container>,
    image_suggestions: Option<Vec<Arc<str>>>,
}

#[derive(thiserror::Error, Debug)]
pub(super) enum CreateIndexTemplateError {
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
            all: true,
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
                status: container.state.map(Status::from),
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

#[derive(thiserror::Error, Debug)]
#[error("Error getting container by id: {0}")]
pub(super) struct GetContainerError(#[from] bollard::errors::Error);
impl GetContainerError {
    fn is_not_found(&self) -> bool {
        matches!(
            self,
            Self(bollard::errors::Error::DockerResponseServerError {
                status_code: 404,
                message: _,
            })
        )
    }
}

async fn get_container(
    docker: &Docker,
    container_id: impl AsRef<str>,
) -> Result<ContainerInspectResponse, GetContainerError> {
    docker
        .inspect_container(container_id.as_ref(), None)
        .await
        .map_err(GetContainerError)
}

/// We only migrate the settings that can be set by our app as the other values are the default values set by the
/// container runtime and changing them can break creating the container depending on the runtime
/// For example podman with cgroupv2 can not set the memory swappiness on a container and will error with
/// "500: crun: cannot set memory swappiness with cgroupv2: OCI runtime error"
fn migrate_host_configuration(container: &HostConfig) -> HostConfig {
    HostConfig {
        port_bindings: container.port_bindings.clone(),
        ..Default::default()
    }
}

fn migrate_configuration(container: &ContainerInspectResponse) -> container::Config<String> {
    container::Config {
        image: container.image.clone(),
        host_config: container
            .host_config
            .as_ref()
            .map(migrate_host_configuration),
        env: container
            .config
            .as_ref()
            .and_then(|configuration| configuration.env.clone()),
        labels: container
            .config
            .as_ref()
            .and_then(|configuration| configuration.labels.clone()),
        ..Default::default()
    }
}

async fn update_container_id(
    connection: libsql::Connection,
    container_id: impl AsRef<str>,
    new_id: impl AsRef<str>,
) -> Result<(), StatementError> {
    let mut statement = connection
        .prepare("UPDATE tokens SET container_id = :new_id WHERE container_id = :old_id")
        .await
        .map_err(StatementError::PrepareStatementError)?;

    let updated_rows = statement
        .execute(named_params! {
            ":old_id": container_id.as_ref(),
            ":new_id": new_id.as_ref(),
        })
        .await
        .map_err(StatementError::ExecuteStatementError)?;

    assert!(updated_rows < 2, "Expected to update one or no container");

    Ok(())
}

pub(super) async fn create(
    State(state): State<TugState>,
    axum_extra::extract::Form(request): axum_extra::extract::Form<CreateRequest>,
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

    let configuration = container::Config::<String> {
        image: Some(String::from(request.image.as_ref())),
        host_config: host_configuration,
        labels: Some(HashMap::from([(label::TAG.to_owned(), String::default())])),
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
    LoadContainerError(#[from] GetContainerError),
    #[error("Error creating token: {0}")]
    CreateTokenError(#[from] token::CreateError),
    #[error("Error hashing token: {0}")]
    HashTokenError(#[from] token::HashError),

    #[error("Error running SQL statement: {0}")]
    StatementError(#[from] StatementError),
}

impl IntoResponse for CreateTokenError {
    fn into_response(self) -> axum::response::Response {
        if matches!(self, Self::LoadContainerError(error) if error.is_not_found()) {
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

#[derive(thiserror::Error, Debug)]
pub(super) enum StatementError {
    #[error("Error preparing SQL statement: {0}")]
    PrepareStatementError(libsql::Error),
    #[error("Error executing SQL statement: {0}")]
    ExecuteStatementError(libsql::Error),
}

pub(super) async fn create_token(
    State(state): State<TugState>,
    Path(container_id): Path<Arc<str>>,
) -> Result<CreateTokenResultTemplate, CreateTokenError> {
    // Check if container exists
    let _container = get_container(&state.docker, &container_id).await?;

    let token = Token::new()?;

    let hash = token.hash()?;

    // Store hash in database
    let mut statement = state
        .connection
        .prepare(
            "INSERT OR REPLACE INTO tokens (container_id, token_hash) VALUES (:container_id, :token_hash)",
        )
        .await
        .map_err(StatementError::PrepareStatementError)?;

    let updated_rows = statement
        .execute(named_params! {
            ":container_id": container_id.as_ref(),
            ":token_hash": hash.as_ref(),
        })
        .await
        .map_err(StatementError::ExecuteStatementError)?;

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

#[derive(thiserror::Error, Debug)]
pub(super) enum UpdateError {
    #[error(transparent)]
    DockerError(#[from] bollard::errors::Error),
    #[error("Expected newly created image to have an id")]
    NoImageId,
    #[error("Container has no name")]
    NoContainerName,
    #[error("Error updating container id: {0}")]
    UpdateContainerIdError(#[from] StatementError),
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
) -> Result<(), UpdateError> {
    //TODO remove clones
    //TODO run update in background and immediately respond with Accepted (202) status code
    let mut locks = state.update_locks.lock().await;
    let update_lock = locks.entry(container_id.clone()).or_default();

    // Don't interfere with running updates. Could make things awkward
    //TODO restrict queue size/only take latest
    let _lock = update_lock.lock().await;

    // Check if containers already exists
    let container = state
        .docker
        .inspect_container(container_id.as_ref(), None::<InspectContainerOptions>)
        .await?;

    let container_name = container
        .name
        .as_ref()
        .ok_or(UpdateError::NoContainerName)?;

    let configuration = migrate_configuration(&container);

    let Some(image_name) = configuration.image.as_ref() else {
        // Updating an image that does not exist is immediately completed
        tracing::debug!("Container has no image. No image to update. Update completed.");
        return Ok(());
    };

    // If the image id is none, then it is invalid and needs to be updated
    let old_image_id = state.docker.inspect_image(&image_name).await?.id;

    // Pull latest image
    tracing::debug!("Pulling image");
    let options = Some(CreateImageOptions {
        // Always include the tag in the name
        from_image: image_name.clone(),
        platform: "linux/amd64".to_string(),
        ..Default::default()
    });

    let mut response_stream = state.docker.create_image(options, None, None);
    while let Some(result) = response_stream.next().await {
        let information = result?;
        tracing::debug!("Create image: {:?}", information.status);
    }
    // Get newly pulled image
    let image = state.docker.inspect_image(image_name.as_ref()).await?;
    tracing::debug!("New image id: {:?}", image.id);
    let new_image_id = image.id.ok_or(UpdateError::NoImageId)?;

    if old_image_id.is_some_and(|id| id == new_image_id) {
        tracing::debug!("Container is up to date");
        return Ok(());
    }

    // Stop container

    tracing::debug!("Stopping container");
    // Returns 304 if the container is not running
    state
        .docker
        .stop_container(&container_id, None::<StopContainerOptions>)
        .await?;

    state
        .docker
        .remove_container(&container_id, None::<RemoveContainerOptions>)
        .await?;

    // Create container
    let options = Some(CreateContainerOptions {
        name: container_name.as_ref(),
        platform: Some("linux/amd64"),
    });

    tracing::debug!("Creating container");

    let new_container = state
        .docker
        .create_container(options, configuration)
        .await?;

    tracing::debug!("Starting container");

    let (start, update) = tokio::join!(
        state
            .docker
            .start_container(&new_container.id, None::<StartContainerOptions<String>>),
        update_container_id(state.connection, container_id, &new_container.id)
    );

    //TODO the other error gets lost if the first one errors
    start?;
    update?;

    tracing::debug!("Started container");
    Ok(())
}

#[derive(thiserror::Error, Debug)]
#[error("Error stopping container: {0}")]
pub(in crate::route) struct StopError(#[from] bollard::errors::Error);

impl IntoResponse for StopError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("Error stopping container: {:?}", self);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

pub(in crate::route) async fn stop_container(
    State(state): State<TugState>,
    Path(container_id): Path<Arc<str>>,
) -> Result<Redirect, StopError> {
    tracing::debug!("Stopping container {container_id}");
    state
        .docker
        .stop_container(&container_id, None::<StopContainerOptions>)
        .await?;

    Ok(Redirect::to("/containers"))
}

#[derive(thiserror::Error, Debug)]
#[error("Error starting container: {0}")]
pub(in crate::route) struct StartError(#[from] bollard::errors::Error);

impl IntoResponse for StartError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("Error starting container: {:?}", self);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

pub(in crate::route) async fn start_container(
    State(state): State<TugState>,
    Path(container_id): Path<Arc<str>>,
) -> Result<Redirect, StartError> {
    tracing::debug!("Starting container {container_id}");
    state
        .docker
        .start_container(&container_id, None::<StartContainerOptions<String>>)
        .await?;

    Ok(Redirect::to("/containers"))
}

#[derive(thiserror::Error, Debug)]
#[error("Error deleting container: {0}")]
pub(in crate::route) struct DeleteError(#[from] bollard::errors::Error);

impl IntoResponse for DeleteError {
    fn into_response(self) -> axum::response::Response {
        tracing::error!("Error deleting container: {:?}", self);
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

pub(in crate::route) async fn delete(
    State(state): State<TugState>,
    Path(container_id): Path<Arc<str>>,
) -> Result<Redirect, DeleteError> {
    tracing::debug!("Deleting container {container_id}");
    state
        .docker
        .remove_container(&container_id, None::<RemoveContainerOptions>)
        .await?;

    Ok(Redirect::to("/containers"))
}
