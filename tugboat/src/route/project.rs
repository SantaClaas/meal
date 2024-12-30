use std::sync::Arc;

use crate::{auth::AuthenticatedUser, TugState};

use askama::Template;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use axum::Form;
use libsql::named_params;
use serde::Deserialize;

#[derive(Deserialize)]
struct Project {
    id: Arc<str>,
    name: Arc<str>,
    image_name: Arc<str>,
}

#[derive(Template, Default)]
#[template(path = "index.html")]
pub(super) struct IndexTemplate {
    projects: Vec<Project>,
}

#[derive(thiserror::Error, Debug)]
pub(super) enum GetIndexError {
    #[error("Error preparing SQL statement: {0}")]
    PrepareStatementError(libsql::Error),
    #[error("Error executing SQL statement: {0}")]
    ExecuteStatementError(libsql::Error),
    #[error("Error reading row: {0}")]
    ReadRowError(libsql::Error),
    #[error("Error deserializing row: {0}")]
    DeserializeRowError(serde::de::value::Error),
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

    let mut statement = state
        .connection
        .prepare("SELECT id, name, image_name FROM projects")
        .await
        .map_err(GetIndexError::PrepareStatementError)?;

    let mut rows = statement
        .query(())
        .await
        .map_err(GetIndexError::ExecuteStatementError)?;

    let mut projects = Vec::new();
    while let Some(row) = rows.next().await.map_err(GetIndexError::ReadRowError)? {
        let project =
            libsql::de::from_row::<Project>(&row).map_err(GetIndexError::DeserializeRowError)?;
        projects.push(project);
    }

    Ok(IndexTemplate { projects }.into_response())
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
        .execute(named_params!(":id": id.clone(), ":name": request.name, ":image_name":request.image_name))
        .await
        .map_err(CreateProjectError::ExecuteStatementError)?;

    //TODO create an update token. Hash it and store the hash in the database
    // Return the token to the user
    Ok(Redirect::to(&format!("/{}", id)))
}

#[derive(Template)]
#[template(path = "project.html")]
pub(super) struct ProjectTemplate {
    project: Project,
}

#[derive(thiserror::Error, Debug)]
pub(super) enum GetProjectError {
    #[error("Error preparing SQL statement: {0}")]
    PrepareStatementError(libsql::Error),
    #[error("Error executing SQL statement: {0}")]
    ExecuteStatementError(libsql::Error),
    #[error("Error deserializing row: {0}")]
    DeserializeRowError(serde::de::value::Error),
}

impl IntoResponse for GetProjectError {
    fn into_response(self) -> axum::response::Response {
        if matches!(
            self,
            GetProjectError::ExecuteStatementError(libsql::Error::QueryReturnedNoRows)
        ) {
            return axum::http::StatusCode::NOT_FOUND.into_response();
        }

        axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

pub(super) async fn get_project(
    State(state): State<TugState>,
    Path(project_id): Path<Arc<str>>,
) -> Result<ProjectTemplate, GetProjectError> {
    let mut statement = state
        .connection
        .prepare("SELECT id, name, image_name FROM projects WHERE id = :id")
        .await
        .map_err(GetProjectError::PrepareStatementError)?;

    // The no row error is currently not handled but could be seen as no error and just not found
    let rows = statement
        .query_row(named_params!(":id": project_id))
        .await
        .map_err(GetProjectError::ExecuteStatementError)?;

    let project =
        libsql::de::from_row::<Project>(&rows).map_err(GetProjectError::DeserializeRowError)?;

    Ok(ProjectTemplate { project })
}
