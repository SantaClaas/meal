use std::{collections::HashMap, sync::Arc};

use axum::{
    http::{self, StatusCode},
    response::IntoResponse,
};
use bollard::{
    container::{
        self, CreateContainerOptions, InspectContainerOptions, RemoveContainerOptions,
        StartContainerOptions, StopContainerOptions,
    },
    image::CreateImageOptions,
    secret::{HostConfig, PortBinding},
    Docker,
};

use tokio::sync::Mutex;
use tokio_stream::StreamExt;

use crate::route::project;

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
pub(super) enum Error {
    #[error(transparent)]
    DockerError(#[from] bollard::errors::Error),
    #[error("Expected newly created image to have an id")]
    NoImageId,
    #[error("Expected container to have an id or name")]
    NoContainerId,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

#[deprecated = "Use generic Manager for updates"]
pub(super) async fn update_melt(
    docker: &Docker,
    update_lock: &Mutex<()>,
) -> Result<UpdateResult, Error> {
    // 1. Check if container already exists
    // 2. Create image
    //    This pulls the latest image
    // 3. Create container
    //    Delete the old one if it exists
    // 4. Start container

    const CONTAINER_NAME: &str = "tugged-melt";
    const IMAGE_NAME: &str = "ghcr.io/santaclaas/meal:main";

    // Don't interfere with running updates. Could make things awkward
    // Technically the current update should be cancelled as it might have pulled the last image which is now outdated
    // but we don't expect so many updates to happen at the same time for this to become a problem
    let Ok(_lock) = update_lock.try_lock() else {
        tracing::debug!("Update already underway");
        return Ok(UpdateResult::AlreadyStarted);
    };

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
    let new_id = image.id.ok_or(Error::NoImageId)?;

    if container_image_id.is_some_and(|id| id == new_id) {
        tracing::debug!("Container is up to date");
        return Ok(UpdateResult::Completed);
    }

    // Stop container if it exists
    if let Some(container) = container {
        let id = container
            .id
            .or(container.name)
            .ok_or(Error::NoContainerId)?;

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

    let configuration = container::Config {
        image: Some(IMAGE_NAME),
        // exposed_ports: Some(HashMap::from([("3000", HashMap::default())])),
        host_config: Some(host_configuration),
        ..Default::default()
    };
    tracing::debug!("Creating container",);

    let response = docker.create_container(options, configuration).await?;

    tracing::debug!("Starting container");
    docker
        .start_container(&response.id, None::<StartContainerOptions<String>>)
        .await?;

    tracing::debug!("Started container");

    Ok(UpdateResult::Completed)
}

/// The container manager manages containers 😉
struct Manager {
    docker: Docker,
    update_locks: Arc<Mutex<HashMap<project::Id, Arc<Mutex<()>>>>>,
}

impl Manager {
    /// 1. Check if container already exists
    /// 2. Create image
    ///    This pulls the latest image
    /// 3. Create container
    ///    Delete the old one if it exists
    /// 4. Start container
    async fn update(
        &self,
        id: project::Id,
        container_name: project::ContainerName,
        image_name: project::ImageName,
    ) -> Result<UpdateResult, Error> {
        let mut locks = self.update_locks.lock().await;
        let update_lock = locks.entry(id).or_default();

        // Don't interfere with running updates. Could make things awkward
        // Technically the current update should be cancelled as it might have pulled the last image which is now outdated
        // but we don't expect so many updates to happen at the same time for this to become a problem
        let Ok(_lock) = update_lock.try_lock() else {
            tracing::debug!("Update already underway");
            return Ok(UpdateResult::AlreadyStarted);
        };

        // Check if containers already exists
        let result = self
            .docker
            .inspect_container(container_name.as_ref(), None::<InspectContainerOptions>)
            .await;

        // 404 Not found is okay
        /// This constant can be inlined in future Rust versions
        const NOT_FOUND: u16 = http::StatusCode::NOT_FOUND.as_u16();
        let container = result.map(Option::Some).or_else(|orrer| match orrer {
            bollard::errors::Error::DockerResponseServerError {
                status_code: NOT_FOUND,
                message: _,
            } => Ok(None),
            other => Err(other),
        })?;

        // If the image is none, then it is invalid and needs to be updated
        let container_image_id = if let Some(container_image) = container
            .as_ref()
            .and_then(|container| container.image.clone())
        {
            let image = self.docker.inspect_image(&container_image).await?;
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
            from_image: image_name.as_ref(),
            platform: "linux/amd64",
            ..Default::default()
        });

        tracing::debug!("Pulling image");

        let mut responses = self.docker.create_image(options, None, None);
        while let Some(result) = responses.next().await {
            let information = result?;
            tracing::debug!("Create image: {:?}", information.status);
        }

        // Get newly pulled image
        let image = self.docker.inspect_image(image_name.as_ref()).await?;
        tracing::debug!("New image id: {:?}", image.id);
        let new_id = image.id.ok_or(Error::NoImageId)?;

        if container_image_id.is_some_and(|id| id == new_id) {
            tracing::debug!("Container is up to date");
            return Ok(UpdateResult::Completed);
        }

        // Stop container if it exists
        if let Some(container) = container {
            let id = container
                .id
                .or(container.name)
                .ok_or(Error::NoContainerId)?;

            tracing::debug!("Stopping container");
            // Returns 304 if the container is not running
            self.docker
                .stop_container(&id, None::<StopContainerOptions>)
                .await?;

            self.docker
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

        let configuration = container::Config {
            image: Some(image_name.as_ref()),
            // exposed_ports: Some(HashMap::from([("3000", HashMap::default())])),
            host_config: Some(host_configuration),
            ..Default::default()
        };
        tracing::debug!("Creating container",);

        let response = self.docker.create_container(options, configuration).await?;

        tracing::debug!("Starting container");
        self.docker
            .start_container(&response.id, None::<StartContainerOptions<String>>)
            .await?;

        tracing::debug!("Started container");
        Ok(UpdateResult::Completed)
    }
}
