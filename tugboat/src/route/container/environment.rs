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

use crate::{
    redirect_to,
    route::container::{migrate_configuration, update_container_id},
    TugState,
};

use super::{get_container, GetContainerError, StatementError};

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

/// Insert adds or replaces the existing variable. It is named after the hashmap insert.
fn insert_variable(configuration: &mut container::Config<String>, key: Arc<str>, value: Arc<str>) {
    let new_variable = format!("{}={}", key, value);

    let Some(variables) = configuration.env.as_mut() else {
        configuration.env = Some(vec![new_variable]);
        return;
    };

    let index = variables
        .iter()
        .position(|variable| variable.starts_with(key.as_ref()));

    let Some(index) = index else {
        variables.push(new_variable);
        return;
    };

    variables[index] = new_variable;
}

enum VariableModification {
    Add { key: Arc<str>, value: Arc<str> },
    Update { key: Arc<str>, value: Arc<str> },
    Delete { key: Arc<str> },
}
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

async fn modify_variable(
    State(state): State<TugState>,
    container_id: Arc<str>,
    modification: VariableModification,
) -> Result<Arc<str>, ModificationError> {
    let container = get_container(&state.docker, &container_id).await?;
    if matches!(modification, VariableModification::Delete { .. })
        && container.config.as_ref().is_none_or(|configuration| {
            configuration
                .env
                .as_ref()
                .is_none_or(|variables| variables.is_empty())
        })
    {
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
        VariableModification::Add { key, value } | VariableModification::Update { key, value } => {
            insert_variable(&mut configuration, key, value)
        }
        VariableModification::Delete { key } => {
            if let Some(variables) = configuration.env.as_mut() {
                let pattern = format!("{}=", key);
                let index = variables
                    .iter()
                    .position(|variable| variable.starts_with(&pattern));
                if let Some(index) = index {
                    variables.swap_remove(index);
                }
            }
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

pub(in crate::route) async fn update(
    state: State<TugState>,
    Path(container_id): Path<Arc<str>>,
    Form(request): Form<UpdateRequest>,
) -> Result<Redirect, ModificationError> {
    let new_container_id = modify_variable(
        state,
        container_id,
        VariableModification::Update {
            key: request.key,
            value: request.value,
        },
    )
    .await?;

    Ok(redirect_to!(
        "/containers/{}/environment/variables",
        new_container_id
    ))
}

#[derive(Deserialize)]
pub(in crate::route) struct DeleteRequest {
    key: Arc<str>,
}

pub(in crate::route) async fn delete(
    state: State<TugState>,
    Path(container_id): Path<Arc<str>>,
    Form(request): Form<DeleteRequest>,
) -> Result<Redirect, ModificationError> {
    let new_container_id = modify_variable(
        state,
        container_id,
        VariableModification::Delete { key: request.key },
    )
    .await?;

    Ok(redirect_to!(
        "/containers/{}/environment/variables",
        new_container_id
    ))
}
