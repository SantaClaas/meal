use std::{num::ParseIntError, sync::Arc};

use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::Form;
use serde::Deserialize;

use crate::{redirect_to, route::error, TugState};

use super::{
    get_container,
    modification::{modify, Modification, ModificationError},
    GetContainerError, Port,
};

#[derive(Template)]
#[template(path = "container/edit.html")]
pub(in crate::route) struct EditTemplate {
    container_id: Arc<str>,
    host_port: Option<Port>,
    container_port: Option<Port>,
}

#[derive(thiserror::Error, Debug)]
pub(in crate::route) enum GetEditError {
    #[error("Error getting container details: {0}")]
    GetContainerError(#[from] GetContainerError),
    #[error("Expected exactly one port binding but got {0}")]
    ExpectedExactlyOnePortBinding(usize),
    #[error("Invalid port binding. Expected format: <container_port>/<protocol> but was {0}")]
    InvalidPortBinding(String),
    #[error("Invalid port format. Port must be a number but was {0}")]
    InvalidPort(String, ParseIntError),
    #[error("Expected one host port but got {0}")]
    ExpectedOneHostPort(usize),
}

impl IntoResponse for GetEditError {
    fn into_response(self) -> Response {
        match self {
            Self::GetContainerError(error) if error.is_not_found() => {
                StatusCode::NOT_FOUND.into_response()
            }
            other => {
                tracing::error!("Error getting container edit page: {other}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

#[axum::debug_handler]
pub(in crate::route) async fn get(
    State(state): State<TugState>,
    Path(container_id): Path<Arc<str>>,
) -> Result<EditTemplate, GetEditError> {
    let container = get_container(&state.docker, &container_id).await?;

    let port_bindings = container
        .host_config
        .as_ref()
        .and_then(|host_config| host_config.port_bindings.as_ref());

    let Some(port_bindings) = port_bindings else {
        return Ok(EditTemplate {
            container_id,
            host_port: None,
            container_port: None,
        });
    };

    tracing::debug!("Port bindings: {port_bindings:?}");

    let mut iterator = port_bindings.iter();
    let (container_port, host_ports) = iterator
        .next()
        .ok_or_else(|| GetEditError::ExpectedExactlyOnePortBinding(port_bindings.len()))?;

    if iterator.next().is_some() {
        return Err(GetEditError::ExpectedExactlyOnePortBinding(
            port_bindings.len(),
        ));
    }

    let (container_port, _protocol) = container_port
        .split_once('/')
        .ok_or_else(|| GetEditError::InvalidPortBinding(container_port.clone()))?;

    let container_port = container_port
        .parse()
        .map_err(|error| GetEditError::InvalidPort(container_port.to_owned(), error))?;

    let Some(host_ports) = host_ports else {
        return Ok(EditTemplate {
            container_id,
            host_port: None,
            container_port: Some(container_port),
        });
    };

    let [host_port] = &host_ports[..] else {
        return Err(GetEditError::ExpectedOneHostPort(host_ports.len()));
    };

    let host_port = host_port
        .host_port
        .as_ref()
        .map(|port| {
            port.parse::<Port>()
                .map_err(|error| GetEditError::InvalidPort(port.to_owned(), error))
        })
        .transpose()?;

    Ok(EditTemplate {
        container_id,
        host_port,
        container_port: Some(container_port),
    })
}

#[derive(Deserialize)]
pub(in crate::route) struct UpdateRequest {
    host_port: Option<Port>,
    container_port: Option<Port>,
}

pub(in crate::route) async fn create(
    state: State<TugState>,
    Path(container_id): Path<Arc<str>>,
    Form(request): Form<UpdateRequest>,
) -> Result<Redirect, error::Response> {
    let new_container_id = modify(
        state,
        container_id,
        Modification::UpdatePortBindings {
            host_port: request.host_port,
            container_port: request.container_port,
        },
    )
    .await?;

    Ok(redirect_to!("/containers/{}/edit", new_container_id))
}
