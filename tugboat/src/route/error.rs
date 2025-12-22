use std::sync::Arc;

use axum::http::StatusCode;

#[derive(askama::Template)]
#[template(path = "error.html")]
pub(in crate::route) struct Template {
    error: Arc<str>,
}

/// Naive error response that should only be returned to authorized users as it exposes internal errors.
pub(in crate::route) struct Response {
    template: Template,
}

impl<E> From<E> for Response
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(error: E) -> Self {
        Self {
            template: Template {
                error: Arc::from(format!("{error:#?}")),
            },
        }
    }
}

impl axum::response::IntoResponse for Response {
    fn into_response(self) -> askama_axum::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.template).into_response()
    }
}
