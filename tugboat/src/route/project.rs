use crate::{auth::AuthenticatedUser, TugState};

use askama::Template;
use axum::extract::State;
use axum::response::{IntoResponse, Redirect};
use bollard::container::ListContainersOptions;
use bollard::secret::ContainerSummary;

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

pub(super) async fn get_index(
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
