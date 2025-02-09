use std::sync::Arc;

use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Form,
};

use bollard::container::{self};

use serde::Deserialize;

use crate::{redirect_to, TugState};

use super::{
    get_container,
    modification::{modify, Modification, ModificationError},
    GetContainerError,
};

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

fn delete_variable(configuration: &mut container::Config<String>, key: Arc<str>) {
    let Some(variables) = configuration.env.as_mut() else {
        return;
    };

    let pattern = format!("{}=", key);
    let index = variables
        .iter()
        .position(|variable| variable.starts_with(&pattern));
    if let Some(index) = index {
        variables.swap_remove(index);
    }
}

pub(super) enum VariableModification {
    Insert { key: Arc<str>, value: Arc<str> },
    Delete { key: Arc<str> },
}

pub(super) fn modify_variable(
    configuration: &mut container::Config<String>,
    modification: VariableModification,
) {
    match modification {
        VariableModification::Insert { key, value } => insert_variable(configuration, key, value),
        VariableModification::Delete { key } => delete_variable(configuration, key),
    }
}

pub(in crate::route) async fn update(
    state: State<TugState>,
    Path(container_id): Path<Arc<str>>,
    Form(request): Form<UpdateRequest>,
) -> Result<Redirect, ModificationError> {
    let new_container_id = modify(
        state,
        container_id,
        Modification::EnvironmentVariable(VariableModification::Insert {
            key: request.key,
            value: request.value,
        }),
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
    let new_container_id = modify(
        state,
        container_id,
        Modification::EnvironmentVariable(VariableModification::Delete { key: request.key }),
    )
    .await?;

    Ok(redirect_to!(
        "/containers/{}/environment/variables",
        new_container_id
    ))
}
