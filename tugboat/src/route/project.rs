use std::fmt::Pointer;
use std::sync::Arc;

use crate::{auth::AuthenticatedUser, TugState};

use askama::Template;
use axum::extract::{Path, State};
use axum::response::{IntoResponse, Redirect};
use axum::Form;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use base64::Engine;
use getrandom::getrandom;
use hkdf::Hkdf;
use libsql::named_params;
use serde::Deserialize;
use sha2::{Digest, Sha256};

#[derive(Deserialize, Clone, Eq, PartialEq, Hash)]
pub(crate) struct Id(Arc<str>);

impl AsRef<str> for Id {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.as_ref().fmt(f)
    }
}

#[derive(Deserialize)]
pub(crate) struct ImageName(Arc<str>);

impl AsRef<str> for ImageName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

impl std::fmt::Display for ImageName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.as_ref().fmt(f)
    }
}

pub(crate) struct ContainerName(Arc<str>);

impl AsRef<str> for ContainerName {
    fn as_ref(&self) -> &str {
        self.0.as_ref()
    }
}

#[derive(Deserialize)]
struct ProjectRow {
    id: Id,
    name: Arc<str>,
    image_name: ImageName,
    token_hash: Option<Arc<[u8; 32]>>,
}

#[derive(Template, Default)]
#[template(path = "index.html")]
pub(super) struct IndexTemplate {
    projects: Vec<ProjectRow>,
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
            libsql::de::from_row::<ProjectRow>(&row).map_err(GetIndexError::DeserializeRowError)?;
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

    Ok(Redirect::to(&format!("/{}", id)))
}

#[derive(thiserror::Error, Debug)]
pub(super) enum GetError {
    #[error("Error preparing SQL statement: {0}")]
    PrepareStatementError(libsql::Error),
    #[error("Error executing SQL statement: {0}")]
    ExecuteStatementError(libsql::Error),
    #[error("Error deserializing row: {0}")]
    DeserializeRowError(serde::de::value::Error),
}

async fn get_row(connection: &libsql::Connection, project_id: Id) -> Result<ProjectRow, GetError> {
    let mut statement = connection
        .prepare("SELECT id, name, image_name, token_hash FROM projects WHERE id = :id")
        .await
        .map_err(GetError::PrepareStatementError)?;

    let rows = statement
        .query_row(named_params!(":id": project_id.as_ref()))
        .await
        .map_err(GetError::ExecuteStatementError)?;

    libsql::de::from_row::<ProjectRow>(&rows).map_err(GetError::DeserializeRowError)
}

#[derive(Template)]
#[template(path = "project.html")]
pub(super) struct ProjectTemplate {
    id: Id,
    name: Arc<str>,
    image_name: ImageName,
    is_token_configured: bool,
    token: Option<Arc<str>>,
}

impl From<ProjectRow> for ProjectTemplate {
    fn from(row: ProjectRow) -> Self {
        ProjectTemplate {
            id: row.id,
            name: row.name,
            image_name: row.image_name,
            is_token_configured: row.token_hash.is_some(),
            token: None,
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub(super) struct GetDetailsError(#[from] GetError);

impl IntoResponse for GetDetailsError {
    fn into_response(self) -> axum::response::Response {
        if matches!(
            self,
            GetDetailsError(GetError::ExecuteStatementError(
                libsql::Error::QueryReturnedNoRows
            ))
        ) {
            return axum::http::StatusCode::NOT_FOUND.into_response();
        }

        axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

pub(super) async fn get_details(
    State(state): State<TugState>,
    Path(project_id): Path<Id>,
) -> Result<ProjectTemplate, GetDetailsError> {
    let project = get_row(&state.connection, project_id.clone()).await?;
    Ok(project.into())
}

#[derive(thiserror::Error, Debug)]
pub(super) enum CreateTokenError {
    #[error("Error loading project: {0}")]
    LoadProjectError(#[from] GetError),
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
        if matches!(
            self,
            CreateTokenError::LoadProjectError(GetError::ExecuteStatementError(
                libsql::Error::QueryReturnedNoRows
            ))
        ) {
            return axum::http::StatusCode::NOT_FOUND.into_response();
        }

        axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

pub(super) async fn create_token(
    State(state): State<TugState>,
    Path(project_id): Path<Id>,
) -> Result<ProjectTemplate, CreateTokenError> {
    // Load project early to bail if it doesn't exist
    let project = get_row(&state.connection, project_id.clone()).await?;

    let mut input_key_material = [0; 128];
    getrandom(&mut input_key_material)?;
    let hkdf =
        Hkdf::<Sha256>::from_prk(&input_key_material).map_err(CreateTokenError::CreateKeyError)?;

    let mut output_key_material = [0; 42];
    hkdf.expand(&[], &mut output_key_material)
        .map_err(CreateTokenError::CreateOutputKeyMaterialError)?;

    let mut hasher = Sha256::new();
    hasher.update(&output_key_material);

    let result = hasher.finalize();

    // Store hash in database
    let mut statement = state
        .connection
        .prepare("UPDATE projects SET token_hash = :token_hash WHERE id = :id")
        .await
        .map_err(CreateTokenError::PrepareStatementError)?;

    let updated_rows = statement
        .execute(named_params!(":token_hash": result, ":id": project.id.as_ref()))
        .await
        .map_err(CreateTokenError::ExecuteStatementError)?;

    assert_eq!(updated_rows, 1, "Expected to update one row");

    let token = BASE64_URL_SAFE_NO_PAD.encode(output_key_material);

    let mut template = ProjectTemplate::from(project);
    template.is_token_configured = true;
    template.token = Some(Arc::from(token));
    Ok(template)
}

#[derive(thiserror::Error, Debug)]
pub(super) enum DeleteError {
    #[error("Error preparing SQL statement: {0}")]
    PrepareStatementError(libsql::Error),
    #[error("Error executing SQL statement: {0}")]
    ExecuteStatementError(libsql::Error),
}

impl IntoResponse for DeleteError {
    fn into_response(self) -> axum::response::Response {
        axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

pub(super) async fn delete(
    State(state): State<TugState>,
    Path(project_id): Path<Id>,
) -> Result<Redirect, DeleteError> {
    let mut statement = state
        .connection
        .prepare("DELETE FROM projects WHERE id = :id")
        .await
        .map_err(DeleteError::PrepareStatementError)?;

    statement
        .execute(named_params!(":id": project_id.as_ref()))
        .await
        .map_err(DeleteError::ExecuteStatementError)?;

    Ok(Redirect::to("/"))
}
