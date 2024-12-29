mod project;

use axum::{routing::get, Router};

use crate::TugState;

pub(crate) fn create_router() -> Router<TugState> {
    Router::new().route("/", get(project::get_index))
}
