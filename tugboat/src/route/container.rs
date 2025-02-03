use std::{collections::HashMap, sync::Arc, time::Duration};

use askama::Template;
use axum::{
    extract::{Path, State},
    http::{self, StatusCode},
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
use sha2::Digest;
use tokio_stream::StreamExt;

use crate::{route::token::Token, TugState};

use super::token;

struct Container {
    id: Arc<str>,
    name: Arc<str>,
    image: Arc<str>,
    state: Option<String>,
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
                state: container.state,
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

#[derive(thiserror::Error, Debug)]
#[error("Error getting container by id: {0}")]
struct GetContainerError(#[from] bollard::errors::Error);
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

pub(in crate::route) mod environment {
    use std::sync::Arc;

    use askama::Template;
    use axum::{
        extract::{Path, State},
        http::StatusCode,
        response::{IntoResponse, Redirect, Response},
        Form,
    };
    use bollard::container::{
        self, CreateContainerOptions, RemoveContainerOptions, StartContainerOptions,
        StopContainerOptions,
    };
    use serde::Deserialize;

    use crate::{redirect_to, TugState};

    use super::{get_container, GetContainerError};

    #[derive(Template)]
    #[template(path = "container/environment.html")]
    pub(in crate::route) struct VariablesTemplate {
        container_id: Arc<str>,
        variables: Option<Vec<(Arc<str>, Arc<str>)>>,
    }

    #[derive(thiserror::Error, Debug)]
    pub(in crate::route) enum GetVariablesError {
        #[error("Error loading container: {0}")]
        LoadContainerError(#[from] GetContainerError),
        #[error("Could not parse environment variable: {0}")]
        ParseError(String),
    }

    impl IntoResponse for GetVariablesError {
        fn into_response(self) -> Response {
            match self {
                Self::LoadContainerError(error) if error.is_not_found() => {
                    StatusCode::NOT_FOUND.into_response()
                }
                Self::LoadContainerError(error) => {
                    tracing::error!("Error loading container: {error}");
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
                other => {
                    tracing::error!("Error getting variables: {other}");
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
            }
        }
    }

    pub(in crate::route) async fn get_variables(
        State(state): State<TugState>,
        Path(container_id): Path<Arc<str>>,
    ) -> Result<VariablesTemplate, GetVariablesError> {
        let container = get_container(&state.docker, &container_id).await?;

        let variables = container
            .config
            .and_then(|configuration| configuration.env)
            .map(|variables| {
                variables
                    .into_iter()
                    .map(
                        |variable| -> Result<(Arc<str>, Arc<str>), GetVariablesError> {
                            variable
                                .split_once('=')
                                .map(|(key, value)| (Arc::from(key), Arc::from(value)))
                                .ok_or_else(|| GetVariablesError::ParseError(variable))
                        },
                    )
                    .collect::<Result<Vec<_>, GetVariablesError>>()
            })
            .transpose()?;

        Ok(VariablesTemplate {
            variables,
            container_id,
        })
    }

    #[derive(Deserialize)]
    pub(in crate::route) struct UpdateRequest {
        key: Arc<str>,
        value: Arc<str>,
    }

    #[derive(thiserror::Error, Debug)]
    pub(in crate::route) enum UpdateError {
        #[error("Error loading container: {0}")]
        LoadContainerError(#[from] GetContainerError),
        #[error("Error updating environment variable: {0}")]
        UpdateError(#[from] bollard::errors::Error),
        #[error("Container was configured without name")]
        NoContainerName,
    }

    impl IntoResponse for UpdateError {
        fn into_response(self) -> Response {
            match self {
                Self::LoadContainerError(error) if error.is_not_found() => {
                    StatusCode::NOT_FOUND.into_response()
                }
                Self::LoadContainerError(error) => {
                    tracing::error!("Error loading container: {error}");
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
                Self::UpdateError(error) => {
                    tracing::error!("Error updating environment variable: {error}");
                    StatusCode::INTERNAL_SERVER_ERROR.into_response()
                }
                UpdateError::NoContainerName => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            }
        }
    }

    pub(in crate::route) async fn update(
        State(state): State<TugState>,
        Path(container_id): Path<Arc<str>>,
        Form(request): Form<UpdateRequest>,
    ) -> Result<Redirect, UpdateError> {
        let container = get_container(&state.docker, &container_id).await?;

        let mut locks = state.update_locks.lock().await;
        let update_lock = locks.entry(container_id.clone()).or_default();

        // Don't interfere with running updates. Could make things awkward
        //TODO restrict queue size
        let _lock = update_lock.lock().await;

        // Stop container
        state
            .docker
            .stop_container(&container_id, None::<StopContainerOptions>)
            .await?;

        state
            .docker
            .remove_container(&container_id, None::<RemoveContainerOptions>)
            .await?;

        // Extract container creation parameters

        let container_name = container.name.ok_or(UpdateError::NoContainerName)?;
        let image_name = container
            .config
            .as_ref()
            .and_then(|configuration| configuration.image.clone());

        let host_configuration = container.host_config;
        let mut environment_variables = container
            .config
            .as_ref()
            .and_then(|configuration| configuration.env.clone());

        // Merge with variables from request
        if let Some(variables) = environment_variables.as_mut() {
            let index = variables
                .iter()
                .position(|variable| variable.starts_with(request.key.as_ref()));

            let new_variable = format!("{}={}", request.key, request.value);
            if let Some(index) = index {
                variables[index] = new_variable;
            } else {
                variables.push(new_variable);
            }
        }

        let labels = container
            .config
            .as_ref()
            .and_then(|configuration| configuration.labels.clone());

        // Create container
        let options = Some(CreateContainerOptions {
            name: container_name.as_ref(),
            platform: Some("linux/amd64"),
        });

        let configuration = container::Config {
            image: image_name,
            host_config: host_configuration,
            env: environment_variables,
            labels,
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

        Ok(redirect_to!(
            "/containers/{container_id}/environment/variables"
        ))
    }

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

    // Load environment variables from file

    let configuration = container::Config::<String> {
        image: Some(String::from(request.image.as_ref())),
        // exposed_ports: Some(HashMap::from([("3000", HashMap::default())])),
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
    #[error("Error preparing SQL statement: {0}")]
    PrepareStatementError(libsql::Error),
    #[error("Error executing SQL statement: {0}")]
    ExecuteStatementError(libsql::Error),
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
    //TODO restrict queue size
    let _lock = update_lock.lock().await;

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
        .as_ref()
        .and_then(|container| container.config.as_ref())
        .and_then(|configuration| configuration.image.clone());

    tracing::debug!("Container image id: {:?}", container_image_id);

    let image_name = if let Some(image_name) = image_name {
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

        Some(image_name)
    } else {
        None
    };

    let container_name = container
        .as_ref()
        .and_then(|container| container.name.clone())
        .ok_or(UpdateError::NoContainerName)?;

    // Stop container if it exists
    if let Some(container) = container.as_ref() {
        let id = container.id.clone().unwrap_or(container_name.clone());

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

    let host_configuration = container
        .as_ref()
        .and_then(|container| container.host_config.clone());

    // Run with same environment variables
    let environment_variables = container.as_ref().and_then(|container| {
        container
            .config
            .as_ref()
            .and_then(|configuration| configuration.env.clone())
    });

    let labels = container.and_then(|container| {
        container
            .config
            .and_then(|configuration| configuration.labels)
    });
    let configuration = container::Config {
        image: image_name,
        // exposed_ports: Some(HashMap::from([("3000", HashMap::default())])),
        host_config: host_configuration,
        env: environment_variables,
        labels,
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
