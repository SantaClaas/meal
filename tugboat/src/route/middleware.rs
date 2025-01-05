use askama_axum::IntoResponse;

use axum::{
    extract::{Path, Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use libsql::{de, named_params, Connection};

use super::token::{self, BadTokenError, Token};

#[derive(thiserror::Error, Debug)]
pub(super) enum AuthorizationError {
    #[error("No authorization header")]
    NoAuthorizationHeader,
    #[error("Bad authorization header value")]
    BadAuthorizationHeader(#[from] BadTokenError),
    #[error("Error preparing statement: {0}")]
    PrepareStatementError(libsql::Error),
    #[error("Error querying database: {0}")]
    QueryError(libsql::Error),
    #[error("Error reading hash from database: {0}")]
    ReadHashError(serde::de::value::Error),
    #[error("Error validating token: {0}")]
    ValidateError(#[from] token::ValidateError),
}

impl IntoResponse for AuthorizationError {
    fn into_response(self) -> Response {
        match self {
            Self::NoAuthorizationHeader => StatusCode::BAD_REQUEST.into_response(),
            Self::BadAuthorizationHeader(_) => StatusCode::BAD_REQUEST.into_response(),
            Self::QueryError(libsql::Error::QueryReturnedNoRows) => {
                // Don't return 404, because that would leak information about the existence of the container
                StatusCode::UNAUTHORIZED.into_response()
            }
            error => {
                tracing::error!("Error authorizing request: {}", error);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

pub(super) async fn require_container_token(
    State(connection): State<Connection>,
    Path(container_id): Path<String>,
    request: Request,
    next: Next,
) -> Result<Response, AuthorizationError> {
    // Check for valid header before attempting database connection
    let header = request
        .headers()
        .get(header::AUTHORIZATION)
        .ok_or(AuthorizationError::NoAuthorizationHeader)?;

    let token: Token = header.try_into()?;

    let mut statement = connection
        .prepare("SELECT token_hash FROM tokens WHERE container_id = :container_id")
        .await
        .map_err(AuthorizationError::PrepareStatementError)?;

    let row = statement
        .query_row(named_params![":container_id": container_id])
        .await
        .map_err(AuthorizationError::QueryError)?;

    let hash = de::from_row::<&[u8]>(&row).map_err(AuthorizationError::ReadHashError)?;

    let hash =
        de::from_row::<token::ContainerHash>(&row).map_err(AuthorizationError::ReadHashError)?;

    if !token.is_valid(hash)? {
        return Ok(StatusCode::UNAUTHORIZED.into_response());
    }

    Ok(next.run(request).await)
}
