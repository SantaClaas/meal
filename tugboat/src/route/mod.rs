mod project;

use axum::{routing::get, Router};

use crate::TugState;

pub(crate) fn create_router() -> Router<TugState> {
    Router::new()
        .route("/", get(project::get_index_page))
        .route("/new", get(project::get_new_page).post(project::create))
        .route("/:project_id", get(project::get_project))
}
