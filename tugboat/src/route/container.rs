use std::{collections::HashMap, sync::Arc, time::Duration};

use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Form,
};
use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine};
use bollard::{
    container::{self, CreateContainerOptions, ListContainersOptions},
    image::CreateImageOptions,
    secret::{HostConfig, PortBinding},
    Docker,
};
use getrandom::getrandom;
use hkdf::Hkdf;
use libsql::{named_params, Connection};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tokio_stream::StreamExt;

use crate::TugState;

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

pub(super) async fn create(
    State(state): State<TugState>,
    Form(request): Form<CreateRequest>,
) -> Result<Redirect, CreateError> {
    // Pull latest image
    tracing::debug!("Pulling image");
    let options = Some(CreateImageOptions {
        // Allways include the tag in the name
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

    let configuration = container::Config {
        image: Some(request.image.as_ref()),
        // exposed_ports: Some(HashMap::from([("3000", HashMap::default())])),
        host_config: host_configuration,
        labels: Some(HashMap::from([(label::TAG, "")])),
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
    CreateTokenError(#[from] getrandom::Error),
    #[error("Error creating key: {0}")]
    CreateKeyError(hkdf::InvalidPrkLength),
    #[error("Error creating output key material: {0}")]
    CreateOutputKeyMaterialError(hkdf::InvalidLength),
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

    let mut input_key_material = [0; 128];
    getrandom(&mut input_key_material)?;
    let hkdf =
        Hkdf::<Sha256>::from_prk(&input_key_material).map_err(CreateTokenError::CreateKeyError)?;

    let mut output_key_material = [0; 42];
    hkdf.expand(&[], &mut output_key_material)
        .map_err(CreateTokenError::CreateOutputKeyMaterialError)?;

    let mut hasher = Sha256::new();
    hasher.update(&output_key_material);

    let hash = hasher.finalize();

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
            ":token_hash": hash.as_slice(),
        })
        .await
        .map_err(CreateTokenError::ExecuteStatementError)?;

    assert_eq!(updated_rows, 1, "Expected to update one row");
    let token = BASE64_URL_SAFE_NO_PAD.encode(output_key_material).into();

    Ok(CreateTokenResultTemplate { token })
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

        // Bail before deleting all containers
        assert!(
            container_ids.len() < 32766,
            "Expected containers to not exceed parameter limit"
        );

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
