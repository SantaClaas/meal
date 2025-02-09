use std::sync::Arc;

use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use bollard::{
    container::{
        CreateContainerOptions, RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
    },
    secret::PortBinding,
};

use crate::{
    route::container::{
        environment::{self},
        migrate_configuration, update_container_id,
    },
    TugState,
};

use super::{get_container, GetContainerError, Port, StatementError};

#[derive(thiserror::Error, Debug)]
pub(in crate::route) enum ModificationError {
    #[error("Error loading container: {0}")]
    LoadContainerError(#[from] GetContainerError),
    #[error("Error updating environment variable: {0}")]
    UpdateError(#[from] bollard::errors::Error),
    #[error("Container was configured without name")]
    NoContainerName,
    #[error("Error updating container id: {0}")]
    UpdateContainerIdError(#[from] StatementError),
}

impl IntoResponse for ModificationError {
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
            Self::NoContainerName => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
            Self::UpdateContainerIdError(error) => {
                tracing::error!("Error updating container id: {error}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

pub(super) enum Modification {
    EnvironmentVariable(environment::VariableModification),
    UpdatePortBindings {
        host_port: Option<Port>,
        container_port: Option<Port>,
    },
}

pub(super) async fn modify(
    State(state): State<TugState>,
    container_id: Arc<str>,
    modification: Modification,
) -> Result<Arc<str>, ModificationError> {
    let container = get_container(&state.docker, &container_id).await?;
    if matches!(
        modification,
        Modification::EnvironmentVariable(environment::VariableModification::Delete { .. })
    ) && container.config.as_ref().is_none_or(|configuration| {
        configuration
            .env
            .as_ref()
            .is_none_or(|variables| variables.is_empty())
    }) {
        return Ok(container_id);
    }

    let mut locks = state.update_locks.lock().await;
    let update_lock = locks.entry(container_id.clone()).or_default();

    // Don't interfere with running updates. Could make things awkward
    //TODO restrict queue size/only take latest
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
    let mut configuration = migrate_configuration(&container);

    match modification {
        Modification::EnvironmentVariable(modification) => {
            environment::modify_variable(&mut configuration, modification);
        }
        Modification::UpdatePortBindings {
            container_port: Some(container_port),
            host_port,
        } => {
            if let Some(port_bindings) = configuration
                .host_config
                .as_mut()
                .and_then(|host_configuration| host_configuration.port_bindings.as_mut())
            {
                let key = format!("{}/tcp", container_port);
                let host_port_bindings = port_bindings.entry(key).or_default();

                if let Some(host_port) = host_port {
                    host_port_bindings.replace(vec![PortBinding {
                        host_ip: Some("0.0.0.0".to_string()),
                        host_port: Some(host_port.to_string()),
                    }]);
                }
            }
        }
        Modification::UpdatePortBindings {
            container_port: None,
            ..
        } => {
            tracing::warn!("No container port specified");
        }
    }

    let container_name = container
        .name
        .ok_or(ModificationError::NoContainerName)?
        .clone();
    // Create container
    let options = Some(CreateContainerOptions {
        name: container_name.as_ref(),
        platform: Some("linux/amd64"),
    });

    tracing::debug!("Creating container",);

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

    Ok(new_container.id.into())
}
