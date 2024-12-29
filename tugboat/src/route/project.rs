use std::sync::Arc;

use crate::{auth::AuthenticatedUser, TugState};

use askama::Template;
use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use axum::Form;
use bollard::container::ListContainersOptions;
use bollard::secret::ContainerSummary;
use libsql::named_params;
use serde::Deserialize;

#[derive(Template, Default)]
#[template(path = "index.html")]
pub(super) struct IndexTemplate {
    containers: Vec<ContainerSummary>,
}

#[derive(thiserror::Error, Debug)]
pub(super) enum GetIndexError {
    #[error("Error getting containers: {0}")]
    DockerError(#[from] bollard::errors::Error),
}

impl IntoResponse for GetIndexError {
    fn into_response(self) -> axum::response::Response {
        axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

pub(super) async fn get_index_page(
    State(state): State<TugState>,
    user: Option<AuthenticatedUser>,
) -> Result<impl IntoResponse, GetIndexError> {
    // Don't show container server information to unauthenticated users
    if user.is_none() {
        return Ok(Redirect::to("/signin").into_response());
    }

    let containers = state
        .docker
        .list_containers(Option::<ListContainersOptions<String>>::None)
        .await?;

    Ok(IndexTemplate { containers }.into_response())
}

#[derive(Template)]
#[template(path = "new project.html")]
pub(super) struct NewProjectTemplate;

pub(super) async fn get_new_page() -> NewProjectTemplate {
    NewProjectTemplate
}

#[derive(Deserialize, Debug)]
pub(super) struct CreateProjectRequest {
    name: Arc<str>,
    #[serde(rename = "image")]
    image_name: Arc<str>,
}

#[derive(thiserror::Error, Debug)]
pub(super) enum CreateProjectError {
    #[error("Error preparing SQL statement: {0}")]
    PrepareStatementError(libsql::Error),
    #[error("Error executing SQL statement: {0}")]
    ExecuteStatementError(libsql::Error),
}

impl IntoResponse for CreateProjectError {
    fn into_response(self) -> axum::response::Response {
        axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

pub(super) async fn create(
    State(state): State<TugState>,
    Form(request): Form<CreateProjectRequest>,
) -> Result<Redirect, CreateProjectError> {
    tracing::debug!("Request: {:?}", request);
    let id = nanoid::nanoid!();
    let mut statement = state
        .connection
        .prepare("INSERT INTO projects (id, name, image_name) VALUES (:id, :name, :image_name)")
        .await
        .map_err(CreateProjectError::PrepareStatementError)?;

    statement
        .execute(named_params!(":id": id, ":name": request.name, ":image_name":request.image_name))
        .await
        .map_err(CreateProjectError::ExecuteStatementError)?;

    Ok(Redirect::to("/"))
}
